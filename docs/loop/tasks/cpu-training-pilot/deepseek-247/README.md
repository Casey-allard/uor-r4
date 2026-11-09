# Real ask-offset checkpoint regression

This directory carries a text-only regression over the sealed Claude-330
question-keyed-read run. The manifest pins:

- the two store archives by SHA-256;
- all eight checkpoint pairs by `model.safetensors` and
  `hamming_read.safetensors` SHA-256; and
- the expected vocabulary, head/depth geometry and ask-selection counts.

The repository does not carry a checkpoint or tarball.

## Reproduce

Download the sealed run into a scratch root. `uor-store` preserves the
complete remote path, so the extracted run lives below
`SCRATCH/results/claude/.../runs-2`:

```sh
SCRATCH=ROOT
RUN_ROOT="$SCRATCH/results/claude/one-model-end-to-end-question-keyed-read-2026-11-08/runs-2"
scripts/pod/uor-store get \
  results/claude/one-model-end-to-end-question-keyed-read-2026-11-08/runs-2/ \
  "$SCRATCH"
```

Extract each seed archive without replacing an existing directory:

```sh
mkdir "$RUN_ROOT/s1/extracted" "$RUN_ROOT/s2/extracted"
tar -xzf "$RUN_ROOT/s1/question-keyed-read-s1.tar.gz" -C "$RUN_ROOT/s1/extracted"
tar -xzf "$RUN_ROOT/s2/question-keyed-read-s2.tar.gz" -C "$RUN_ROOT/s2/extracted"
```

Build and run:

```sh
cargo build --release --offline --locked -p uor-r4-cpu-train \
  --bin cpu-ask-arm-regression
target/release/cpu-ask-arm-regression \
  --manifest docs/loop/tasks/cpu-training-pilot/deepseek-247/real-checkpoint-regression.json \
  --root "$RUN_ROOT" \
  --workers 8 \
  --out NEW-REPORT.json
```

The runner verifies the archive hashes before auditing any checkpoint. An
archive failure aborts before report generation and exits `1`. It then checks
all checkpoint hashes and summaries in parallel. An unreadable checkpoint or
checkpoint-hash mismatch is written as a per-arm source failure and exits `1`.
An observed summary that differs from its expectation writes the report and
exits `2`. Existing output paths are refused.

## What the readings mean

Each cell compares the zero-offset question code with the trained code against
all 16 relation keys. `trained_target_nearest` counts the cells where the
question offset makes its intended key the nearest code.

| Arms | Seed 1 / seed 2 | Known interpretation |
|---|---:|---|
| B1-a | 64 / 64 | Found-copy pull at the scored rows trains selection |
| B1-a0 | 9 / 1 | Task loss alone does not train the offsets |
| B2-a | 0 / 3 | A pull at the scored row does not train question selection |
| B2-aq | 64 / 64 | A pull at the question row trains selection |

These are relation-key geometry readings. They do not measure carry from the
question row to `H4`, answer quality, B2, serving or capability.
