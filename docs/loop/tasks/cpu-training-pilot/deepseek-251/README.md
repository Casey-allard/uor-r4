# CPU carry-hold checkpoint audit

`cpu-carry-hold-audit` is a read-only check for Claude-334 checkpoints that
contain the held ask write. It validates the saved hold representation and
reports whether the learned gain or type-write row moved.

```sh
CARGO_TARGET_DIR=target/deepseek-carry-hold-audit \
cargo build --release --offline --locked -p uor-r4-cpu-train \
  --bin cpu-carry-hold-audit

target/deepseek-carry-hold-audit/release/cpu-carry-hold-audit \
  --checkpoint CHECKPOINT \
  --expected-model-sha256 MODEL_SHA256 \
  --expected-hamming-sha256 HAMMING_SHA256 \
  --expected-hold-rows 8 \
  --out NEW-REPORT.json
```

The checkpoint must have:

- ask tables at depths 2 and 6;
- `hold_rows` as one U32 value;
- `d2.hold` as finite F32 `[vocab]`;
- `d2.weight` as finite F32 `[2, width]`.

The report includes:

- hold row count, gain nonzero count, mean, minimum, maximum and mean absolute
  value;
- the type-write row's nonzero count and mean absolute value;
- the same 16-relation ask-selection summary as the trained-offset audit.

A zero gain is a `g`-not-trained reading. A moved gain with failing carry is a
carry-path reading. Neither result by itself is a B2 or serving result.
