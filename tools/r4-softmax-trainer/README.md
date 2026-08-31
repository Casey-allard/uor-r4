# R4 causal-softmax trainer (#1014 / #1017)

This package contains the bounded offline training paths authorized by issues
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014) and
[#1017](https://github.com/UOR-Foundation/uor-r4/issues/1017). They train and
continue an ordinary causal-softmax Llama-family model, export it in the
existing Rust loaders' Hugging Face format, and freeze evidence before each
sealed test is opened. They contain no teacher, trace-distillation,
comparison-arm, resonance, or routing experiment.

The model is fixed: vocabulary 4096; hidden width 288; six layers; six query
and six KV heads; head width 48 (twelve R4 blocks); SwiGLU width 768; context
256; tied embedding/head; bias-free RMSNorm/RoPE/SwiGLU; learned Q/K/V/O; and
ordinary stable complete-prefix softmax. The exact parameter count is
7,155,360. Float multiplication, allocation, autograd, and MPS are intentional
offline operations. This package is not the exact/table runtime and does not
establish geometric advantage, reasoning, or release readiness.

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
7.15M-parameter checkpoint again, or tune its learning rate. The next research
campaign must be a separately frozen parameter-capacity increase over the same
qualified attention and Rust evidence path. External training hardware is
allowed only if that new contract requires it. See the
[#1017 record](../../docs/r4_softmax_quality_capacity_continuation_1017.md).

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

The run contract binds `pyproject.toml`, `uv.lock`, and every package source
file by a sorted BLAKE3 tree, in addition to dependency versions. MPS is
mandatory; the trainer refuses CPU fallback and a six-hour campaign overrun.

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

## Local verification

These checks do not download data or train a model:

```bash
"$ROOT/venv/bin/python" -m unittest discover -s "$TRAINER/tests" -v
"$ROOT/venv/bin/python" -m compileall -q "$TRAINER/src" "$TRAINER/tests"
```

See [NOTICE.md](NOTICE.md) for the pinned MIT `llama2.c` precedent and
TinyStories attribution. No upstream source, weights, environments, corpus
bytes, token stores, checkpoints, or exports are vendored.
