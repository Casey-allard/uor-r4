# R4 offline trainers

This package contains the bounded offline training paths authorized by issues
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) and
[#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017), plus the frozen,
preflight-recorded [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019)
parameter-capacity campaign, and the bounded [#954](https://github.com/UOR-Foundation/uor-r4/issues/954)
grounding fine-tune, frozen source-span-pointer successor, and independently
frozen source-relative relation-head, attended-relation, and joint-candidate
successors, followed by the frozen paired-query binding rung. The package also
contains #973's isolated source-free `R4GroupAddressedRetentionLMV1` campaign,
its independently frozen `R4GroupAddressedRetentionDecoderV1CpuRecovery`, and
the compact matched `R4RetainedLanguagePathV1` generalization rung, its
terminal `R4PairedH4PromptCapacityV1` successor, and the frozen
`R4DirectRetainedReadoutLanguagePathV1` campaign.
The causal-softmax and grounding paths train
and continue ordinary causal-softmax
Llama-family models, export them in the existing Rust loaders' Hugging Face
format, and freeze evidence before each sealed test is opened. They contain no
teacher, trace-distillation, resonance, or routing experiment. The #973 path is
a separate matched-arm experiment and consumes only the frozen tokenizer and
training-store data—not #1017 weights or traces.

The #1014/#1017 model is fixed at vocabulary 4096, hidden width 288, six
layers, six query and KV heads, head width 48 (twelve R4 blocks), SwiGLU width
768, context 256, and exactly 7,155,360 parameters. #1019 preserves every one
of those fields except decoder depth, which is frozen at twelve layers and
exactly 13,130,784 parameters. Both use tied embedding/head, bias-free
RMSNorm/RoPE/SwiGLU, learned Q/K/V/O, and ordinary stable complete-prefix
softmax. Float multiplication, allocation, and autograd are intentional offline
operations. This package is not the exact/table runtime and does not establish
geometric advantage, reasoning, or release readiness.

## Offline execution selection

Before any substantial deterministic trainer run, predeclare a small set of
materially plausible, scientifically eligible backend/thread/worker plans and
benchmark one representative unit from each. Select the measured-fast stable plan that preserves the
declared result and fits memory; neither one core nor the maximum worker count is
an acceptable silent default. Record backend/BLAS provider, PyTorch
intra/inter-op settings, process count, utilization, representative timing, and equivalence
evidence in the freeze. MPS is eligible only when the scientific contract allows
it; CUDA requires explicit issue scope. Offline acceleration does not change the
CPU/table-native deployed-runtime target. The normative rule is in the root
[`AGENTS.md`](../../AGENTS.md#long-run-discipline-process-amendment-2026-08-06).

## Completed #1045 role-tagged associative ladder

The canonical contract and append-only evidence record are
[`docs/r4_role_tagged_associative_curriculum_1045.md`](../../docs/r4_role_tagged_associative_curriculum_1045.md).
This campaign starts from #1043's ordinary initialization and open construction
population only. It never reads #1043's fitted artifact or sealed evaluation
payloads. Its create-once lifecycle is:

```bash
export UOR_MODEL_STORE="/absolute/path/to/the/shared/.uor-models"
ROOT="$UOR_MODEL_STORE/research/issue-1045-role-tagged-associative"
SOURCE_ROOT="$UOR_MODEL_STORE/research/issue-1043-position-kv-binding"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"

uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" prepare-role-tagged-associative --source-root "$SOURCE_ROOT"
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" preflight-role-tagged-associative
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" run-role-tagged-associative
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" verify-role-tagged-associative
```

Preparation binds the input and trainer implementation identities. Preflight
must establish the causal role oracle, the disposable 32-row overfit, and an
eligible Apple Accelerate CPU plan before R1 may run. A positive preflight is
mechanics and resource-admission evidence, not a held-out associative-attention
result. R1 alone decides `OPEN_MQAR_LEARNED` versus
`OPEN_MQAR_NOT_LEARNED` on assignment-disjoint open MQAR and its destructive
controls. Role-off is a separate, non-gating attribution check; attention-off
is unavailable under the frozen mechanics. Even a positive R1 authorizes only
the next open English-transfer rung—not generation, geometric advantage,
softmax replacement, exact lowering, reasoning, product, or release claims.
The executed R1 stopped `OPEN_MQAR_NOT_LEARNED`: construction reached
`65,500/65,536`, while assignment-disjoint development reached
`7,137/8,192`. Its binding next action was the separately scoped #1047 port of
the released Zoology cell and loader, followed by #1049's measured-wall
execution; exact evidence and CIDs are in the canonical records.

## #1047 released Zoology MQAR control

The credited source port and frozen contract are documented in
[`docs/r4_zoology_mqar_control_1047.md`](../../docs/r4_zoology_mqar_control_1047.md).
Its create-once lifecycle is exposed as a module:

```bash
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"
cd "$TRAINER"
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_control prepare \
  /absolute/run/root --source-root /absolute/1043/root \
  --predecessor-root /absolute/1045/root
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_control preflight \
  /absolute/run/root
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_control run \
  /absolute/run/root
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_control verify \
  /absolute/run/root
```

The executed C0 passed exact source goldens and disposable overfit, but no
frozen CPU plan met the 900-second admission wall: the all-core 8-thread plan
projected `959.212581 s`. The immutable result is `NOT_RUN_PREFLIGHT`, CID
`blake3:b453abccc6ae0db9cc186c791aba268555dc0e75fe687c994e940254b0ac9ef6`.
No training artifact or scientific attention verdict was produced.

## #1049 measured-wall Zoology MQAR execution

The frozen continuation and executed result are documented in
[`docs/r4_zoology_mqar_measured_wall_1049.md`](../../docs/r4_zoology_mqar_measured_wall_1049.md).
#1049 preserved the #1047 source-derived cell, loader, RNG namespaces, batch
64, optimizer, schedule, and thresholds, changing only execution provenance
and the CPU wall from 900 to 1,200 seconds.

C0 repeated exact source mechanics and the disposable `128/128` overfit. The
all-core 8-thread plan completed C1's 64 epochs in `195.201318 s` total wall.
Construction reached `32,758/32,768` (`99.969482%`), while development peaked
at `999/4,096` (`24.389648%`) and finished at `980/4,096` (`23.925781%`). The
immutable verdict is `SCALED_SOURCE_CALIBRATION_MISS`; C2 and binding
permutation are `NOT_RUN_C1_MISS`. Result CID is
`blake3:9b36540d81d0967a3f7e2ccabed80900d31c904b6c747d9ba0d539b325b13373`.

A read-only diagnosis found that `4,089/4,096` predictions were one of the four
values present in the row, but key-specific binding remained at four-choice
chance. The pinned executable release used 100,000 construction rows, batch
512, and the best of four frozen rates, rather than #1049's 8,192 rows, batch
64, and one rate. The only authorized next copied-control decision is a fresh,
released-configuration reproduction contract, now frozen in #1050. These
commands and #1049's create-once root are retained as evidence and are not a
rerun interface. Do not tune #1049 or proceed to C2 from its result.

## #1050 released-configuration Zoology reproduction

The canonical contract and executed result are documented in
[`docs/r4_zoology_release_reproduction_1050.md`](../../docs/r4_zoology_release_reproduction_1050.md).
This sibling package preserves #1049's immutable implementation and reproduces
the executable Figure-2 T=64 configuration and training semantics: 100,000
training rows, 3,000 source test/early-stop rows, batch 512, seed 123, the
released DataLoader RNG trajectory, cosine schedule, and frozen source learning
rates. CPU placement and query-only tied-head projection are the two declared
adaptations; a direct full-versus-query-only loss and gradient test passed.

Its create-once lifecycle is:

```bash
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"
ROOT="/absolute/issue-1050-root"
PREDECESSOR_ROOT="/absolute/issue-1049-root"
cd "$TRAINER"
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_release prepare \
  "$ROOT" --predecessor-root "$PREDECESSOR_ROOT"
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_release preflight \
  "$ROOT"
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_release run \
  "$ROOT"
PYTHONPATH=src .venv/bin/python -m r4_softmax_trainer.zoology_release verify \
  "$ROOT"
```

One/four/eight-thread preflight selected four intra-op CPU threads. The first
frozen source rate passed the strict source threshold at epoch 20 with
`11,900/12,000` (`99.1666667%`) top-1 and NLL
`0.05124610455830892` in `577.834602 s`; the remaining rates were not run
because the source early stop fired. Result CID is
`blake3:bd16d012c01262ffb8c5197e4cf316c6fee1d722cf0700a0048386180a8122e0`;
artifact CID is
`blake3:163cf3e5375b3e721fa7a826acdb2dfc809e5989209b03fb2a3eea3e3d5459e9`.
The split has zero full-row overlap but is not assignment-disjoint and is
evaluated each epoch; it is held out only from gradient updates.

The create-once root is evidence, not a rerun or tuning interface. This result
rules out a broken copied cell but does not isolate which reduced-versus-
released contract difference caused #1049's miss. The next decision is one
freshly initialized transfer to the exact open #1045 bytes under the positive
training semantics; R4, W8, English, generation, and broad sweeps stay out of
that transfer issue. Final audit also limits the current lifecycle claim: its
implementation CID does not cover `pyproject.toml`/`uv.lock`, and an
interruption immediately after a passing-epoch checkpoint is not guaranteed to
preserve first-pass early stop. The successor must close both provenance and
resume gaps before its own run; neither occurred in #1050's uninterrupted run.

## #1059 coherent R4 inference integration

[#1059](https://github.com/UOR-Foundation/uor-r4/issues/1059) reuses the retained
qualified #1050 artifact and test container for fixed-weight inference. It
performs no training or optimizer updates and leaves the #1057 continuation
artifact and checkpoint preserved. The existing #1050 evidence root, including
its source envelopes, model and dataset, is a prerequisite; these commands do
not rerun the source reproduction.

Choose absolute source and new evidence paths, then export the native
`R4SpinFrameAtlas` mapping for all 8,192 token IDs. The exporter writes the
canonical H4 frame sidecar, token leaves and causal prefix witnesses into a new
directory. Preparation binds these files, the source artifacts and the current
implementation/dependency closure without scoring a fitted model:

```bash
INFERENCE_REPO="$(git rev-parse --show-toplevel)"
INFERENCE_TRAINER="$INFERENCE_REPO/tools/r4-softmax-trainer"
INFERENCE_SOURCE="/absolute/retained/issue-1050-zoology-release-reproduction"
INFERENCE_ROOT="/absolute/new/issue-1059-zoology-r4-inference"
INFERENCE_FRAMES="$INFERENCE_ROOT/frames"

mkdir -p "$INFERENCE_ROOT"
cargo run --release --locked --manifest-path "$INFERENCE_REPO/Cargo.toml" \
  -p uor-r4-core --bin r4-zoology-frame-export -- "$INFERENCE_FRAMES"
PYTHONPATH="$INFERENCE_TRAINER/src" "$INFERENCE_TRAINER/.venv/bin/python" \
  -m r4_softmax_trainer.zoology_r4_inference prepare "$INFERENCE_ROOT" \
  --source-root "$INFERENCE_SOURCE" --frames-root "$INFERENCE_FRAMES"
```

Use the package's locked Python environment. Commit the implementation and
record its preparation identity in the issue before fitted-model scoring.
Then run the matched integration and its independent replay as separate Python
processes:

```bash
PYTHONPATH="$INFERENCE_TRAINER/src" "$INFERENCE_TRAINER/.venv/bin/python" \
  -m r4_softmax_trainer.zoology_r4_inference run "$INFERENCE_ROOT"
PYTHONPATH="$INFERENCE_TRAINER/src" "$INFERENCE_TRAINER/.venv/bin/python" \
  -m r4_softmax_trainer.zoology_r4_inference verify "$INFERENCE_ROOT"
```

Both primary arms use unchanged learned tensors and tied weights, complete
model evaluation mode, canonical test rows `0..2999`, and batch size 512.
Only the three `test_*` tensor values are loaded; labels reach the scorer, not
the attention adapter. The new canonical logits digests do not reuse #1050's
order-dependent shuffled-test digest. The primary criteria require the source
count of `11,900/12,000`, identical plain/R4 top-1 decisions, and the frozen
logit, attention and NLL tolerances recorded in preparation.

Only after primary integration passes does one control deliberately mismatch
the source transport frames using the causal-prefix cyclic permutation while
retaining true source encodings, payloads, positions, weights and support. Its
causal/work integrity and recall loss are reported separately. A weak or
invalid control does not discard a preserved primary integration result or
authorize another fit; it does not establish H4 superiority.

Execution uses four CPU threads, one inter-op thread and one process. The run
and fresh-process replay share a 900-second allowance and a 4 GiB peak-RSS
ceiling. Finite batch progress, `result.json` and `replay.json` retain output
digests, metrics, source/frame identities and actual work counts. Replay must
reproduce the complete inference evidence exactly while all bound files and
learned tensors remain unchanged. Start markers prevent an interrupted run or
replay from silently renewing its budget; resource interruption is incomplete
evidence rather than a model failure.

## Terminal #973 group-retention and decoder paths

The canonical contract and evidence log are
[`docs/r4_group_addressed_retention_973.md`](../../docs/r4_group_addressed_retention_973.md).
The Rust `r4-group-geometry-export` binary emits the bound exact-H4,
cyclic-120, scrambled-H4, and prime-leaf artifact. This package exposes
`prepare-group-retention --geometry <artifact>` and
`preflight-group-retention --backend mps`. The preflight completed at
`UNAVAILABLE_FRAME_POPULATION_OR_LOCAL_BUDGET`: geometry, population,
reachability, gradients, memory, equal work, and held-out sealing passed, but
timing and disposable learning smoke failed. Main optimization and held-out
model scoring are `NOT_RUN`. The exact signed envelopes are preserved in
`docs/`. The retained commands are not an
authorized retry or tuning surface, and the terminal package exposes no
held-out-open API. It was not retried or reinterpreted as a model negative.

The independently frozen
[`R4GroupAddressedRetentionDecoderV1CpuRecovery`](../../docs/r4_group_addressed_retention_decoder_cpu_recovery_973.md)
then used deterministic Apple Accelerate CPU BLAS with one process and a
configured PyTorch intra/inter-op thread count of four. It completed all 512 construction steps in
`438.117083 s`. State-off on the disjoint construction-validation partition
lost `0.967227` nats and 182
top-1 hits, qualifying a bounded causal retained-attention component. The exact
complete-decoder recipe is not promoted because aggregate validation CE worsened `8.371911 -> 8.976155`;
scrambled transport was `0.033049` nats better, so no H4-specific advantage is
claimed. That next action later completed as `R4RetainedLanguagePathV1` below.
There is no authorized retry, CUDA path, sweep, or C1-SB6.

## Frozen #973 retained language-path rung

[`R4RetainedLanguagePathV1`](../../docs/r4_retained_language_path_v1_973.md)
is one from-scratch two-arm comparison: the qualified exact-H4 retained cell
versus ordinary full-prefix causal RoPE Q/K/V softmax. Both arms have exactly
252,160 learned parameters, two width-48 blocks, four 12-wide heads, equal
23,040-value full-context K/V state, tied output storage, and identical
5,241,600-decision optimizer dose. The data freezer copies only CID-bound
nonsealed slices from #1019 plus its tokenizer and the inherited canonical
geometry; it reads no checkpoint, weight, teacher logit, sealed confirmation,
or heldout reveal.

The lifecycle is deliberately short and ordered:

```bash
export UOR_MODEL_STORE="/absolute/path/to/the/shared/.uor-models"
ROOT="$UOR_MODEL_STORE/research/issue-973-retained-language-path-v1"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"

uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" prepare-language-path
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" probe-language-path
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" run-language-path
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" generate-language-path
```

Preparation is create-once. The probe measures deterministic four-thread and
eight-thread Apple Accelerate CPU, concurrent isolated two-thread workers, and
deterministic MPS, then binds the fastest admitted aggregate plan. CUDA is
forbidden. `run-language-path --resume` may only continue the byte-identical
same trajectory after interruption; it is not a scientific retry.

The frozen fit completed at `RETAINED_LANGUAGE_PATH_PASS`: the retained arm
generalized, remained load-bearing under its state-off intervention, and was
competitive with the matched ordinary decoder. `generate-language-path` is
the separately frozen positive branch. It reads only the copied tokenizer and
geometry plus the immutable retained result/artifact, executes five public
prompts for at most 64 selected tokens, and records an exact fresh-load replay
without training or opening train, validation, source, or sealed data. Its
create-once result is a local autonomous-decoding smoke, not a coherence,
reasoning, H4-superiority, lowering, browser, or release claim.

## Terminal #973 paired-H4 prompt-capacity rung

[`R4PairedH4PromptCapacityV1`](../../docs/r4_paired_h4_prompt_capacity_973.md)
changes only the qualified retained cell's token addressing: token zero remains
the canonical exact-H4 identity, while every other token receives a reversible
two-coordinate radix address, one coordinate per decoder layer. The V1
parameter count, state ledger, projections, gates, training slice/order, seed,
and optimizer dose remain fixed; frozen V1 is evaluated without retraining.

The create-once lifecycle is:

```bash
export UOR_MODEL_STORE="/absolute/path/to/the/shared/.uor-models"
ROOT="$UOR_MODEL_STORE/research/issue-973-paired-h4-prompt-capacity-v1"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"

uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" prepare-paired-h4-prompt-capacity
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" probe-paired-h4-prompt-capacity
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" run-paired-h4-prompt-capacity
```

The run completed on Apple Accelerate CPU with four threads and terminated
`PAIRED_H4_PROMPT_CAPACITY_FAIL`. Fresh heldout NLL/top-1 slightly improved
over V1, but independent prompt gain was `0.0062477543` for the candidate
versus `0.0063672952` for V1, and candidate wins were `282/512` against the
required 308. State-off collapsed to zero; replay, causal, and forbidden-read
audits passed. The candidate is not promoted. Preserve V1 and independently
freeze the prompt-state-to-logit readout seam. There is no authorized
generation retry, parameter sweep, CUDA path, or C1-SB6.

## Terminal #973 direct retained-readout rung

[`R4DirectRetainedReadoutLanguagePathV1`](../../docs/r4_direct_retained_readout_prompt_capacity_973.md)
keeps qualified V1's exact-H4 recurrence, parameter/state counts, data/order,
seed, optimizer dose, and tied output matmul fixed. It exposes the two
already-computed retained layer outputs only at the final head:
`E @ (N(h) + g*N(a1+a2))`, fixed `g=1` candidate versus `g=0` V1 control.

Its create-once lifecycle was:

```bash
export UOR_MODEL_STORE="/absolute/path/to/the/shared/.uor-models"
ROOT="$UOR_MODEL_STORE/research/issue-973-direct-retained-readout-v1"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"

uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" prepare-direct-retained-readout
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" probe-direct-retained-readout
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" run-direct-retained-readout
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" verify-direct-retained-readout
```

Apple Accelerate CPU with four threads completed the sole 2,730-step trajectory
in `1,313.037 s`. Prompt gain improved from `0.0076304198` to `0.0215897894`,
wins from `313/512` to `343/512`, and fresh held-out NLL/top-1 from
`3.9010778353` / `29.632946%` to `3.7374367989` / `31.542433%`. State removal
cost `1.1234286047` nats. Exact replay and a separate-process verifier passed.
The terminal is `DIRECT_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL` because both
frozen gain floors were missed. There is no generation, retry, widened readout,
gain tuning, CUDA path, exact lowering, or C1-SB6.

## Terminal #973 layerwise-normalized retained-readout rung

`R4LayerwiseNormalizedRetainedReadoutLanguagePathV1` preserves every qualified
V1 budget and uses the exact zero-parameter formula
`E @ [N(h) + (g/sqrt(2))*(N(a1)+N(a2))]`, fixed `g=1` versus equal-work `g=0`.
Its create-once lifecycle was:

```bash
export UOR_MODEL_STORE="/absolute/path/to/the/shared/.uor-models"
ROOT="$UOR_MODEL_STORE/research/issue-973-layerwise-normalized-retained-readout-v1"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"

uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" prepare-layerwise-normalized-readout
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" probe-layerwise-normalized-readout
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" run-layerwise-normalized-readout
uv run --offline --project "$TRAINER" r4-softmax-trainer \
  --root "$ROOT" verify-layerwise-normalized-readout
```

Apple Accelerate CPU with four threads completed the sole 2,730-step trajectory
in `1,447.764 s`. Prompt gain was `0.0286980210` versus matched V1 at
`0.0073316237` (delta `0.0213663973`), with `339/512` wins. Fresh held-out
NLL/top-1 improved to `3.7126411677` / `31.661826%` from
`3.8850003883` / `29.728138%`; state removal cost `1.3495375637` nats and
20,595 decisions. Exact replay and all `13/13` separate-process verifier
comparisons passed. The terminal is
`LAYERWISE_NORMALIZED_RETAINED_READOUT_PROMPT_CAPACITY_PARTIAL` because both
frozen gain floors were missed. Result and verification CIDs are
`blake3:35396bd6e64fc2c0bc7d86a84cc9e212ed913ce28e5353f5f2b8212b4cf2c532`
and `blake3:3f316541dbab8061ed5ba891bf6a47ef22c55bca21fba01f6f97dbb3cb8497aa`.

This valid miss ends the parameter-free readout ladder. #973 must next freshly
freeze learned associative binding/readout. There is no gain tuning, `g=2`,
third normalization variant, retry, widened readout, generation, reasoning,
CUDA path, or lowering run from this result; #954 remains blocked.

## Current measured boundary

#1014 established load-bearing ordinary causal attention at this learned
R4/Spin scope, but failed its complete quality Definition of Done. #1017 then
continued that exact model, optimizer state, tokenizer, split discipline, and
runtime path for one frozen 7,324-step campaign. It reached `149,995,520`
cumulative training tokens and selected development NLL
`1.580241072373312`. Enabled-only Python/Rust parity passed with identical
top-1 and maximum logit delta `0.0000057220458984375`; all six layers and every
causal/external audit passed. The one-time fresh sealed NLL was
`1.5727521962806827`, failing the strict `<1.50` gate, while subject-or-scene
retention and normalized replay both passed `5/5`.

The #1017 result is negative solely on NLL. Do not rerun it, extend this
7.15M-parameter checkpoint again, or tune its learning rate. It remains the
current working coherent-generation prototype: `r4 generate --prompt "..."`
defaults to `$UOR_MODEL_STORE/research/issue-1017/export` (or the same path
under `.uor-models` when unset). #1019 is an optional
quality-capacity improvement: a fresh seed-1019, twelve-layer,
13,130,784-parameter run of exactly
16,800 steps and 275,251,200 tokens over the same qualified attention and Rust
evidence path. The exact population, 400-step fixed-sequence overfit, and
random-export all-twelve-layer Rust preflight parity passed. The signed MPS gate stopped
`UNAVAILABLE_HARDWARE_BUDGET` on time: its `20.66 h` safety projection exceeded
the `8 h` ceiling, while memory passed at `21.03%`. That terminal applies only
to the frozen offline PyTorch/MPS implementation. Full training, final parity,
reveal, generation, and replay remain `NOT_RUN`. A single isolated exact-shape
MPS fast-path test (10 warmup plus 40 measured steps) combined fused AdamW with
deferred logging and measured `4.485223 s/step`, slower than the signed
`3.491307 s/step`; `fused=True` was removed immediately. This is a bounded
fast-path negative, not a model result. #1019 closed without a full run. #954's
grounding fine-tune, positive-diagonal cosine pointer, source-relative relation
probe, attended-relation adapter, joint-candidate adapter, and paired-query
binding adapter have completed.
C1-SB3 transferred most relations through the six attention layers but failed
its exact gate. C1-SB4 then executed its independently frozen full-source,
record-level structured-margin run: exact records were `70/126` fit and
`35/63` sealed, with perfect positive-group recall and negative specificity
`394/478` and `197/239`. It stopped before Rust/checkpoint/development/product
and must not be retried. A question-ignoring ` is inside ` rule reproduces the
entire published aggregate exactly. C1-SB5
`R4PairedQueryCandidateMatrixV1` then fit all `56/56` paired records but reached
only `14/28` sealed. Row-swap equivariance was bit-exact; pair-mean-query and
attention-off controls were each `0/28`. Products remained unopened, no
checkpoint or binding head was emitted, and Rust parity, development, and
product evaluation were `NOT_RUN`. Terminal
`FAIL_PAIRED_QUERY_BINDING_PREFLIGHT` retires the rung without retry. This is
bounded source-backed attention evidence only, not generation, reasoning,
correctness, or a source-free runtime. UOR's deployed architecture/runtime remains CPU-native; Apple Accelerate/BLAS
and MPS are local offline accelerators only; CUDA and external GPU execution
are out of scope. The MPS stop is not a model-quality negative,
leaves the full-scale capacity hypothesis untested, and does not revoke the
established attention result. See the
[#1017 record](../../docs/r4_softmax_quality_capacity_continuation_1017.md) and
[#1019 frozen contract](../../docs/r4_softmax_parameter_capacity_1019.md) plus
its [observed preflight](../../docs/r4_softmax_parameter_capacity_preflight_1019_raw.json).
The #954 evidence is in the
[#954 record](../../docs/r4_grounded_correctness_954.md) and
[C1-SB4 aggregate](../../docs/r4_joint_candidate_margin_954_raw.json), followed
by the [C1-SB5 aggregate](../../docs/r4_paired_query_binding_954_raw.json).

For fast local #1017 inference on Apple Silicon, build the Rust CLI with
`--features local-inference-accelerate`. The observed four-token comparison
preserved generated IDs, output CID, and attention-audit CID while reducing
internal generation from `3.060506042 s` to `0.116236875 s`. This is ordinary
Apple CPU BLAS and carries distinct backend provenance; it is not a CUDA path
or a change to the exact portable runtime contract.

## Historical bounded #954 grounding fine-tune

`finetune-grounding` is the next product-facing step over the completed #1017
model. It runs one fixed 384-step MPS fine-tune with a fresh AdamW optimizer;
there is no sweep. Every example uses this exact contract:

```text
Use only the context. Copy one exact contiguous answer span from the context. If the context does not answer the question, write ABSTAIN. If the context gives conflicting answers, write CONTRADICTION.
Context:
{source}
Question:
{question}
Answer:
```

The corpus deterministically balances supported, unsupported, and conflicting
examples. Loss applies only to answer and EOS tokens; prompt and padding targets
are masked. Three reserved prompts remain absent from training and development
for the subsequent Rust product check. The run reuses the #1017 tokenizer,
six-layer model shape and standard Hugging Face export. It also writes a
Python prefix-logit fixture as a diagnostic artifact; no separate Rust parity
campaign is required for this product gate. At the conservative measured `3.491307 s/step`, the 384 optimizer
steps are about 22 minutes 21 seconds before the small development evaluations
and export.

From any worktree root, point at the shared model store and run:

```bash
export UOR_MODEL_STORE=/Users/casey.allard/uor-r4/.uor-models
PYTHONPATH="$PWD/tools/r4-softmax-trainer/src" \
  "$UOR_MODEL_STORE/research/issue-1014/venv/bin/python" \
  -m r4_softmax_trainer finetune-grounding
```

The defaults read
`$UOR_MODEL_STORE/research/issue-1017/export` and write
`$UOR_MODEL_STORE/research/issue-954`. If interrupted after a checkpoint, rerun
the same command with `--resume`. Completion exports `issue-954/export` and the
enabled-only Python prefix fixture; grounded generation remains `NOT_RUN` until
the Rust product command passes all three reserved prompts.

## C1-SB1 source-span pointer result

`train-source-span-pointer` implemented the frozen
`R4SourceSpanPointerV1`: each exact subject and punctuation-terminated source
sentence was encoded independently through the immutable #1017 model, then a
positive 288-lane diagonal weighted cosine selected a source span while three
explicit logits selected answer, abstain, or conflict. The two-invocation gate
first emitted one 12-record overfit head, required a Rust `r4 answer` report,
and admitted the sole full fit only after Python/Rust score parity.

The preflight passed `12/12`; maximum absolute Rust/Python score and logit
deltas were `1.234420776e-7` and `1.428717041e-6`, respectively, against the
frozen `0.01` tolerance. The one 256-step fit then failed its development gate:
answer classification was `89/128` (`69.53125%`), abstention `114/128`
(`89.0625%`), conflict `117/128` (`91.40625%`), and supported span selection
`121/128` (`94.53125%`), all below the required `95%`.

Terminal: `FAIL_SOURCE_SPAN_POINTER_DEVELOPMENT_GATE_STOP`. No final pointer
artifact was emitted, and all three committed product probes remain `NOT_RUN`.
Do not rerun, tune, or use the preflight head as a product head. The retained
state-capture and parity seams subsequently supported the independently frozen
C1-SB2 source-relative relation probe below.
See the [#954 record](../../docs/r4_grounded_correctness_954.md) and
[structured C1-SB1 result](../../docs/r4_source_span_pointer_954_raw.json).

## C1-SB2 source-relative relation-head result

`train-source-relation-head` implemented
`R4SourceRelativeRelationHeadV1`. Each exact sentence is encoded together with
the exact question, ending at the question-mark token. The immutable #1017
executor supplies that final width-288 normalized R4/Spin state to a fixed
`288 -> 32 ReLU -> 1` probe. Strict positive logits identify supporting
relations; exact duplicate text collapses before deterministic exact-copy,
abstain, or contradiction selection.

The zero-training census passed over 3,360 construction and 420 lexically
disjoint development records. The mandatory cheap gate then fit all 12 records
from two construction families exactly but failed all six answer decisions in
the two unseen families. Sealed answer, abstain, and conflict decisions were
`0/6`, `3/4`, and `1/2`; positive recall was `5/12`, negative specificity
`14/20`, and copied-span accuracy `0/6`. Terminal:
`FAIL_MATCHED_TRANSFER_PREFLIGHT_STOP`.

No Python/Rust parity report, 512-step full fit, final head, development result,
or product evaluation exists. Do not rerun or tune the frozen probe. The
preserved trainer/runtime seam is for replay and for a future independently
frozen mechanism that trains relation semantics into the representation. See
the [structured C1-SB2 result](../../docs/r4_source_relation_head_954_raw.json).

## C1-SB3 attended-relation adapter result

`prepare-attended-relation` and `train-attended-relation-preflight` implemented
`R4AttendedRelationAdapterV1`: rank-eight Q/K/V/O LoRA in every one of the six
attention layers, fixed tied-token yes/no scoring, and no learned head. Corrected
evaluation showed bounded transfer—sealed positive recall `73/76`, negative
specificity `234/239`, and only the 24 attention tensors changed—but the exact
fit/sealed record gates failed at `124/126` and `56/63`. No merged checkpoint,
Rust parity, full fit, development, or product reveal followed. Do not retry
this independent-candidate BCE mechanism. See the
[corrected C1-SB3 aggregate](../../docs/r4_attended_relation_adapter_954_raw.json).

## C1-SB4 joint-candidate structured-margin result

`prepare-joint-candidate-margin` commits the fresh data, tokenizer census, and
four opaque product records without training. The separately consumed
`train-joint-candidate-margin-preflight` used complete-source candidate prompts,
collapsed exact duplicate groups, and optimized one record-level margin across
positive and negative groups. Its only admitted configuration was seed 9544,
270 MPS updates, seven complete records per update, and a step-eight/600-second
wall gate.

The run completed within budget but failed exactly: `70/126` fit and `35/63`
sealed records, positive groups `126/126` and `63/63`, negative specificity
`394/478` and `197/239`, and same-source query relocation false. All 24 Q/K/V/O
targets changed and no non-attention tensor changed. Rust parity, checkpoint,
development, reversal, and product are `NOT_RUN`; the products remain unopened.
A question-ignoring ` is inside ` rule reproduces every published aggregate
count exactly. C1-SB5 subsequently consumed that finding by coupling multiple
questions over one source. Do not rerun C1-SB4.

The already-consumed invocation was:

```bash
PYTHONPATH="$PWD/tools/r4-softmax-trainer/src" \
  "$UOR_MODEL_STORE/research/issue-1014/venv/bin/python" \
  -c 'from r4_softmax_trainer.cli import main; main()' \
  --root "$UOR_MODEL_STORE/research/issue-954/joint-candidate-margin" \
  train-joint-candidate-margin-preflight \
  --predecessor "$UOR_MODEL_STORE/research/issue-1017/export"
```

The started/result markers intentionally reject a second invocation. See the
[#954 record](../../docs/r4_grounded_correctness_954.md) and
[C1-SB4 aggregate](../../docs/r4_joint_candidate_margin_954_raw.json).

## C1-SB5 paired-query binding result

`prepare-paired-query-binding` committed a fresh fit/sealed population plus four
opaque product-pair commitments without optimization or product access. The sole
`train-paired-query-binding-preflight` execution used paired questions over each
exact source, rank-8 Q/K/V/O LoRA in all six attention layers, and an asymmetric
rank-32 query/candidate binding head. Its 120 MPS steps fit `56/56` pairs, but
only `14/28` independently sealed pairs transferred. Identity-aligned row-swap
traces were bit-exact; pair-mean-query and attention-off controls each fell to
`0/28`. The terminal was `FAIL_PAIRED_QUERY_BINDING_PREFLIGHT`.

The already-consumed invocations were:

```bash
PYTHONPATH="$PWD/tools/r4-softmax-trainer/src" \
  "$UOR_MODEL_STORE/research/issue-1014/venv/bin/python" \
  -m r4_softmax_trainer \
  --root "$UOR_MODEL_STORE/research/issue-954/paired-query-binding" \
  prepare-paired-query-binding \
  --predecessor "$UOR_MODEL_STORE/research/issue-1017/export"

PYTHONPATH="$PWD/tools/r4-softmax-trainer/src" \
  "$UOR_MODEL_STORE/research/issue-1014/venv/bin/python" \
  -m r4_softmax_trainer \
  --root "$UOR_MODEL_STORE/research/issue-954/paired-query-binding" \
  train-paired-query-binding-preflight \
  --predecessor "$UOR_MODEL_STORE/research/issue-1017/export"
```

The exclusive start marker forbids a second training invocation. The product
population remained unopened, and the negative branch emitted no checkpoint or
binding-head artifact and ran no Rust parity, development, or product stage.
Retire C1-SB5 without retry. See the
[#954 record](../../docs/r4_grounded_correctness_954.md) and
[C1-SB5 aggregate](../../docs/r4_paired_query_binding_954_raw.json).

## Isolated environment

Python is frozen to 3.12 and every dependency is exact in `pyproject.toml` and
`uv.lock`. Keep the environment beside all bulk artifacts in the repository's
ignored model store:

```bash
ROOT="$(git rev-parse --show-toplevel)/.uor-models/research/issue-1014"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"
uv venv "$ROOT/venv" --python 3.12
UV_PROJECT_ENVIRONMENT="$ROOT/venv" uv sync --frozen --project "$TRAINER"
CLI="$ROOT/venv/bin/r4-softmax-trainer"
```

The #1014/#1017 run contracts bind `pyproject.toml`, `uv.lock`, and every
package source file by a sorted BLAKE3 tree, in addition to dependency
versions. Their MPS-only limits remain historical campaign constraints. #1019
instead used its own eight-hour backend-admission gate. MPS stopped
`UNAVAILABLE_HARDWARE_BUDGET` on time (`20.66 h > 8 h`) while memory passed at
`21.03%`. That result applies only to the frozen offline implementation. The
subsequent fused-AdamW/deferred-logging fast path was slower (`4.485223` versus
signed `3.491307 s/step`), so #1019 closed without a full run. #954's grounding
fine-tune, source-span pointer, source-relative relation probe,
attended-relation adapter, joint-candidate adapter, and paired-query binding
adapter also closed negative; none is rerun.
CUDA and external GPU execution are out of scope.

## One-way campaign

Run these stages in order. `--root` defaults to the same ignored path shown
above.

```bash
"$CLI" --root "$ROOT" download
"$CLI" --root "$ROOT" prepare
"$CLI" --root "$ROOT" smoke
# Run the Rust 32-token smoke export parity gate here.
"$CLI" --root "$ROOT" train
# `--resume` may resume only the identical frozen run contract.
"$CLI" --root "$ROOT" reveal
```

`download` verifies the pinned `roneneldan/TinyStories` revision, exact byte
length, and SHA-256. `prepare` scans the complete source and assigns each raw
canonical story before tokenization using the full 32-byte
`BLAKE3(story_bytes)` interpreted as a big-endian integer modulo 100. Buckets
0–89/90–94/95–99 are train/development/sealed test. The 4096-id ByteLevel BPE
is trained on train stories only with null normalizer and post-processor,
dense IDs, and explicit BOS/EOS insertion by this tool.

Training may expose 30,000,000 train tokens and 250,000 development tokens.
The sealed budget is strict: 249,880 scored-store token IDs plus five globally
lowest-test-story prompts of 24 IDs each equals 250,000. The full-source scan
records the test population and ordered story-CID digest. A training-view
manifest contains only tokenizer/train/dev artifacts and commitments to sealed
files; smoke and main training never open the full dataset manifest or any
`sealed-test/` path.

`smoke` must reduce mean loss by at least 80% on exactly 64 fixed sequences in
at most five minutes. A pass exports a separate `smoke/export/` snapshot and a
two-arm 32-token enabled/attention-off logit fixture for the mandatory Rust
loader parity check. It does not authorize a research claim.

`train` performs one campaign (no sweep), selects minimum development NLL,
exports `config.json`, `model.safetensors`, and `tokenizer.json`, then freezes
the checkpoint/export tree in `selection/selection-manifest.json` while the
test status is still `UNOPENED`. Only `reveal` may then open sealed files. It
scores enabled and post-O-projection-zeroed attention over identical complete
test blocks, requiring enabled NLL at most 1.50 and an attention-off penalty of
at least 0.10 nats/token. It also emits:

- `sealed-test/prompts.json`: the first 24 content tokens of the five globally
  lowest full-story CIDs;
- `reveal/python-prefix-logits.json`: enabled/off full 4096-logit vectors for
  the first 32 stored test IDs, with the 0.005 Rust parity tolerance;
- `reveal/reveal-manifest.json`: the qualification input binding selection,
  data/split/weights identities, sealed inputs, metrics, and reference logits.

Autonomous continuations are deliberately owned by the Rust local generator's
explicit seeded sampler, R4/Spin transport audit, and replay contract. Python
does not grade or substitute preview generations.

## Frozen #1017 continuation lifecycle

The #1017 evidence has already been consumed and frozen. The commands below
document the one-way protocol; they do not authorize a rerun, a second reveal,
new generations, or replacement rubric decisions.

```bash
CONTINUATION_ROOT="$(git rev-parse --show-toplevel)/.uor-models/research/issue-1017"
PREDECESSOR_ROOT="$(git rev-parse --show-toplevel)/.uor-models/research/issue-1014"

"$CLI" --root "$CONTINUATION_ROOT" prepare-continuation \
  --predecessor-root "$PREDECESSOR_ROOT"
"$CLI" --root "$CONTINUATION_ROOT" continue --resume
"$CLI" --root "$CONTINUATION_ROOT" verify-continuation-training
"$CLI" --root "$CONTINUATION_ROOT" admit-enabled-parity \
  --rust-qualification /absolute/path/to/enabled-rust-qualification.json
"$CLI" --root "$CONTINUATION_ROOT" reveal-continuation
"$CLI" --root "$CONTINUATION_ROOT" finalize-continuation \
  --rubric /absolute/path/to/independent-five-record-rubric.json
```

`prepare-continuation` constructs the fresh disjoint training, development,
and denied confirmation population from the immutable #1014 predecessor.
`continue --resume` may resume only the identical frozen run contract.
`verify-continuation-training` reproduces the nonsealed training-view evidence
while confirmation access remains denied. `admit-enabled-parity` binds the sole
enabled-only 32-token Rust qualification. `reveal-continuation` irreversibly
opens and scores the fresh confirmation exactly once.

After the already archived Rust generator and replay reports exist,
`finalize-continuation --rubric` validates and binds those ten reports, the
opened reveal, and the independent five-record rubric into create-once final
evidence. Finalization executes no model, generates no token, and opens no
population or reveal.

## Frozen #1019 parameter-capacity lifecycle

#1019 changes only decoder depth from six to twelve layers. Its fixed training
population, 64-sequence overfit test, random-export Python/Rust parity check,
200-step hardware probe, full run, selected export, all-twelve-layer Rust
qualification, one-time reveal, five seeds 3019 through 3023, and normalized
replays were all `NOT_RUN` at contract freeze. The first four stages have since
run as recorded below. Do not treat a hardware stop or partial checkpoint as
language-quality evidence.

The observed preflight has since passed the exact population, 400-step
fixed-sequence overfit, and random-export all-twelve-layer Rust preflight parity gates.
Its signed MPS probe stopped `UNAVAILABLE_HARDWARE_BUDGET` because the
`20.66 h` safety projection exceeded `8 h`; memory passed at `21.03%`. That
terminal applies only to the frozen offline implementation. Full training,
final parity, reveal, generation, and replay remain `NOT_RUN`. The subsequent
fused-AdamW/deferred-logging fast path was slower (`4.485223` versus signed
`3.491307 s/step`), so #1019 closed without a full run. #954's six bounded
source-grounding mechanisms through C1-SB5 subsequently closed negative. CUDA
and external GPU execution are out of scope. See the
[#1019 observed preflight](../../docs/r4_softmax_parameter_capacity_preflight_1019_raw.json).

The Rust qualifier has a separate shape-bound campaign mode. After a #1019
export and Python development-prefix fixture exist, it rejects the legacy
six-layer shape and requires all twelve layers. The already-consumed MPS
preflight sequence was:

```bash
CAPACITY_ROOT="$(git rev-parse --show-toplevel)/.uor-models/research/issue-1019"
CONTINUATION_ROOT="$(git rev-parse --show-toplevel)/.uor-models/research/issue-1017"
SOURCE="$(git rev-parse --show-toplevel)/.uor-models/research/issue-1014/raw/TinyStoriesV2-GPT4-train.txt"
TRAINER="$(git rev-parse --show-toplevel)/tools/r4-softmax-trainer"

uv venv "$CAPACITY_ROOT/venv" --python 3.12
UV_PROJECT_ENVIRONMENT="$CAPACITY_ROOT/venv" uv sync --frozen --project "$TRAINER"
CAPACITY_CLI="$CAPACITY_ROOT/venv/bin/r4-softmax-trainer"

"$CAPACITY_CLI" --root "$CAPACITY_ROOT" prepare-capacity \
  --predecessor-root "$CONTINUATION_ROOT" --source "$SOURCE"
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" smoke-capacity --backend mps

cargo run --release --offline --bin r4 -- r4-softmax-local-qualify \
  --model "$CAPACITY_ROOT/preflight/smoke-export" \
  --python-prefix-logits "$CAPACITY_ROOT/preflight/python-capacity-smoke-prefix.json" \
  --campaign issue-1019 \
  --workers 4 \
  --json-output "$CAPACITY_ROOT/preflight/rust-capacity-smoke-input.json"

"$CAPACITY_CLI" --root "$CAPACITY_ROOT" admit-capacity-smoke \
  --rust-qualification "$CAPACITY_ROOT/preflight/rust-capacity-smoke-input.json"
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" probe-capacity --backend mps
```

That MPS probe wrote `UNAVAILABLE_HARDWARE_BUDGET` on time, so do not rerun its
create-once population, smoke, Rust parity, admission, or MPS probe stages.
Preserve the passed smoke admission. CUDA and external GPU execution are out of
scope. The one fused-AdamW/deferred-logging fast-path test was slower than the
signed baseline, so do not tune or launch the #1019 full run. #1019 is closed;
do not reopen recurring optimization or broad research gates. The later #954
grounding campaigns are recorded separately and are not reasons to reopen #1019.

After a locally admitted full run produces an export, continue with:

```bash
cargo run --release --offline --bin r4 -- r4-softmax-local-qualify \
  --model "$CAPACITY_ROOT/export" \
  --python-prefix-logits "$CAPACITY_ROOT/qualification/python-capacity-prefix-logits.json" \
  --campaign issue-1019 \
  --workers 4 \
  --json-output "$CAPACITY_ROOT/qualification/rust-capacity-prefix-input.json"

"$CAPACITY_CLI" --root "$CAPACITY_ROOT" admit-capacity-parity \
  --rust-qualification "$CAPACITY_ROOT/qualification/rust-capacity-prefix-input.json"
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" verify-capacity-training
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" reveal-capacity \
  --baseline-1017-root "$CONTINUATION_ROOT"
```

`--resume` may continue only the authenticated byte-identical frozen run and
backend; elapsed wall time remains monotone across an interruption. After a
positive NLL reveal, the following is the exact generation and replay stage.
It refuses a nonempty generation directory before the first irreversible run.

```bash
(
set -euo pipefail
: "${CAPACITY_ROOT:?set CAPACITY_ROOT}"
: "${CAPACITY_CLI:?set CAPACITY_CLI}"
REVEAL="$CAPACITY_ROOT/reveal/capacity-reveal-result.json"
GENERATION_ROOT="$CAPACITY_ROOT/generations"
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" verify-capacity-generation-ready >/dev/null
jq -e '.terminal == "PASS_CAPACITY_NLL_ADVANCE_GENERATION"' "$REVEAL" >/dev/null
if [ -e "$GENERATION_ROOT" ]; then
  echo "generation evidence already exists; refusing overwrite" >&2
  exit 1
fi
mkdir -p "$GENERATION_ROOT/replay"

for INDEX in 0 1 2 3 4; do
  SEED=$((3019 + INDEX))
  PROMPT="$(jq -r --argjson index "$INDEX" '.prompts[$index].prompt_text' "$REVEAL")"
  PRIMARY="$GENERATION_ROOT/prompt-$INDEX-seed-$SEED.json"
  REPLAY="$GENERATION_ROOT/replay/prompt-$INDEX-seed-$SEED.json"
  test ! -e "$PRIMARY" && test ! -e "$REPLAY"
  cargo run --release --offline --bin r4 -- r4-softmax-local-generate \
    --model "$CAPACITY_ROOT/export" --prompt "$PROMPT" \
    --max-new-tokens 128 --seed "$SEED" --json-output "$PRIMARY"
  cargo run --release --offline --bin r4 -- r4-softmax-local-generate \
    --model "$CAPACITY_ROOT/export" --prompt "$PROMPT" \
    --max-new-tokens 128 --seed "$SEED" --json-output "$REPLAY"
done

RUBRIC="$CAPACITY_ROOT/review/capacity-human-rubric.json"
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" prepare-capacity-rubric \
  --output "$RUBRIC"
)
```

The template command first validates the exact five primary/replay pairs. An
independent reviewer must replace every `REVIEW_...` placeholder with `PASS`
or `FAIL` and a nonempty reason without changing the bound index, story CID,
seed, or response text. Then freeze terminal evidence exactly once:

```bash
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" finalize-capacity \
  --rubric "$CAPACITY_ROOT/review/capacity-human-rubric.json"
```

Running a Rust qualification, reveal, generation, or finalization command does
not by itself establish a passing result; only the create-once terminal
evidence does.

The authoritative model, population, optimization, admission, reveal,
positive/negative branch, and nonclaim definitions live in the
[#1019 record](../../docs/r4_softmax_parameter_capacity_1019.md) and its
[structured contract](../../docs/r4_softmax_parameter_capacity_1019_raw.json).
The consumed preflight evidence lives in the
[#1019 observed preflight](../../docs/r4_softmax_parameter_capacity_preflight_1019_raw.json).

## Local verification

These checks do not download data or train a model:

```bash
"$ROOT/venv/bin/python" -m unittest discover -s "$TRAINER/tests" -v
"$ROOT/venv/bin/python" -m compileall -q "$TRAINER/src" "$TRAINER/tests"
```

See [NOTICE.md](NOTICE.md) for the pinned MIT `llama2.c` precedent and
TinyStories attribution. No upstream source, weights, environments, corpus
bytes, token stores, checkpoints, or exports are vendored.
