# R4 causal-softmax trainer (#1014 / #1017 / #1019 / #954)

This package contains the bounded offline training paths authorized by issues
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) and
[#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017), plus the frozen,
preflight-recorded [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019)
parameter-capacity campaign, and the bounded [#954](https://github.com/UOR-Foundation/uor-r4/issues/954)
grounding fine-tune plus its frozen source-span-pointer successor. They train
and continue ordinary causal-softmax
Llama-family models, export them in the existing Rust loaders' Hugging Face
format, and freeze evidence before each sealed test is opened. They contain no
teacher, trace-distillation, comparison-arm, resonance, or routing experiment.

The #1014/#1017 model is fixed at vocabulary 4096, hidden width 288, six
layers, six query and KV heads, head width 48 (twelve R4 blocks), SwiGLU width
768, context 256, and exactly 7,155,360 parameters. #1019 preserves every one
of those fields except decoder depth, which is frozen at twelve layers and
exactly 13,130,784 parameters. Both use tied embedding/head, bias-free
RMSNorm/RoPE/SwiGLU, learned Q/K/V/O, and ordinary stable complete-prefix
softmax. Float multiplication, allocation, and autograd are intentional offline
operations. This package is not the exact/table runtime and does not establish
geometric advantage, reasoning, or release readiness.

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
grounding fine-tune and positive-diagonal cosine pointer have also completed as
bounded negatives. The next #954 mechanism must be frozen independently before
use; it is not a retry of either revealed run. UOR's deployed architecture/runtime remains CPU-native; Apple Accelerate/BLAS
and MPS are local offline accelerators only; CUDA and external GPU execution
are out of scope. The MPS stop is not a model-quality negative,
leaves the full-scale capacity hypothesis untested, and does not revoke the
established attention result. See the
[#1017 record](../../docs/r4_softmax_quality_capacity_continuation_1017.md) and
[#1019 frozen contract](../../docs/r4_softmax_parameter_capacity_1019.md) plus
its [observed preflight](../../docs/r4_softmax_parameter_capacity_preflight_1019_raw.json).

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
state-capture and parity seams may support a future independently frozen
source-relative relation/entailment head that preserves exact copy semantics.
See the [#954 record](../../docs/r4_grounded_correctness_954.md) and
[structured C1-SB1 result](../../docs/r4_source_span_pointer_954_raw.json).

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
fine-tune and source-span pointer also closed negative; neither is rerun.
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
`3.491307 s/step`), so #1019 closed without a full run. #954's two bounded
source-grounding mechanisms subsequently closed negative. CUDA and external GPU execution are out of scope. See the
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
