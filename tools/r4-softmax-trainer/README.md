# R4 causal-softmax trainer (#1014)

This is the one bounded, offline training path authorized by issue
[#1014](https://github.com/UOR-Foundation/uor-r4/issues/1014). It trains an
ordinary causal-softmax Llama-family model from random initialization, exports
it in the existing Rust loaders' Hugging Face format, and freezes evidence
before the sealed test is opened. It contains no teacher, trace-distillation,
comparison-arm, resonance, or routing experiment.

The model is fixed: vocabulary 4096; hidden width 288; six layers; six query
and six KV heads; head width 48 (twelve R4 blocks); SwiGLU width 768; context
256; tied embedding/head; bias-free RMSNorm/RoPE/SwiGLU; learned Q/K/V/O; and
ordinary stable complete-prefix softmax. The exact parameter count is
7,155,360. Float multiplication, allocation, autograd, and MPS are intentional
offline operations. This package is not the exact/table runtime and does not
establish geometric advantage, reasoning, or release readiness.

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

## Local verification

These checks do not download data or train a model:

```bash
"$ROOT/venv/bin/python" -m unittest discover -s "$TRAINER/tests" -v
"$ROOT/venv/bin/python" -m compileall -q "$TRAINER/src" "$TRAINER/tests"
```

See [NOTICE.md](NOTICE.md) for the pinned MIT `llama2.c` precedent and
TinyStories attribution. No upstream source, weights, environments, corpus
bytes, token stores, checkpoints, or exports are vendored.
