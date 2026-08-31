# R4 causal-softmax trainer (#1014 / #1017 / #1019)

This package contains the bounded offline training paths authorized by issues
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) and
[#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017), plus the frozen,
preflight-recorded [#1019](https://github.com/UOR-Foundation/uor-r4/issues/1019)
parameter-capacity campaign. They train and continue ordinary causal-softmax
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
7.15M-parameter checkpoint again, or tune its learning rate. #1019 is the sole
successor: a fresh seed-1019, twelve-layer, 13,130,784-parameter run of exactly
16,800 steps and 275,251,200 tokens over the same qualified attention and Rust
evidence path. The exact population, 400-step fixed-sequence overfit, and
random-export all-twelve-layer Rust preflight parity passed. The signed MPS gate stopped
`UNAVAILABLE_HARDWARE_BUDGET` on time: its `20.66 h` safety projection exceeded
the `8 h` ceiling, while memory passed at `21.03%`. Full training, final parity,
reveal, generation, and replay remain `NOT_RUN`. Only the deterministic
single-CUDA `f32` fallback may proceed after explicit owner authorization for
external compute and any spend. The MPS stop is not a model-quality negative,
leaves the full-scale capacity hypothesis untested, and does not revoke the
established attention result. See the
[#1017 record](../../docs/r4_softmax_quality_capacity_continuation_1017.md) and
[#1019 frozen contract](../../docs/r4_softmax_parameter_capacity_1019.md) plus
its [observed preflight](../../docs/r4_softmax_parameter_capacity_preflight_1019_raw.json).

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
`21.03%`. The only permitted fallback is one pinned deterministic single-CUDA
`f32` environment, after explicit owner authorization for external compute and
any spend; TF32 remains disabled.

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
replays are all `NOT_RUN` at contract freeze. Do not launch the full campaign
until the preflight report admits a backend under the eight-hour ceiling. Do
not start a paid job without explicit owner approval, and do not treat a
hardware stop or partial checkpoint as language-quality evidence.

The observed preflight has since passed the exact population, 400-step
fixed-sequence overfit, and random-export all-twelve-layer Rust preflight parity gates.
Its signed MPS probe stopped `UNAVAILABLE_HARDWARE_BUDGET` because the
`20.66 h` safety projection exceeded `8 h`; memory passed at `21.03%`. Full
training, final parity, reveal, generation, and replay remain `NOT_RUN`. The
only allowed next path is the deterministic single-CUDA `f32` fallback after
explicit owner authorization for external compute and any spend. See the
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
Reuse the passed smoke admission and run only the probe in the one
contract-permitted deterministic CUDA `f32` environment after explicit owner
authorization for external compute and any spend. The create-once smoke must
not be repeated. Do not improvise a CPU or mixed-precision fallback.

```bash
# Requires explicit owner authorization and the pinned deterministic CUDA f32 environment.
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" probe-capacity --backend cuda

# Run only if the signed CUDA probe returns PASS_HARDWARE_ADMISSION.
"$CAPACITY_CLI" --root "$CAPACITY_ROOT" train-capacity --backend cuda

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

Use `train-capacity --backend cuda` only when the signed CUDA probe passed.
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
