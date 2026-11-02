# One-model-end-to-end instruments

This standalone crate writes M-world v2 panels and evaluates an exported integer discrete model on a
CPU. It is not in the workspace and it does not train or modify model weights.

## `heldout-panel`

```sh
cargo run --release --offline --bin heldout-panel -- \
  out=NEW_DIR tokenizer=TOKENIZER.json cell=PHRASING,VALUE \
  conversations=400 seed=1 protocol=2
```

`cell=dev,dev` holds prompt phrasing out. `cell=train,train` is a generated control. The writer emits
`tokens.u16`, `response_mask.u8` and a hash-bound `manifest.json`.

## `m2-evidence`

The input paths are explicit. `train-role` is required with a reference index and records whether it
is canonical TRAIN or a generated control:

```sh
cargo run --release --offline --bin m2-evidence -- \
  artifact=MODEL.d11 \
  tokenizer=TOKENIZER.json \
  panel-tokens=PANEL/tokens.u16 \
  panel-mask=PANEL/response_mask.u8 \
  panel-manifest=PANEL/manifest.json \
  train-tokens=TRAIN/tokens.u16 \
  train-mask=TRAIN/response_mask.u8 \
  train-manifest=TRAIN/manifest.json \
  train-role=generated_control \
  max-targets=1024 \
  out=NEW_REPORT.json
```

The report contains:

- artifact schema, table count, hashes and shape;
- panel and reference manifest, tokenizer, content hashes and corpus identity;
- target admission, top-1 and top-8 overall;
- opening versus continuation and prompt-overlap, value-masked prompt-template, first-8-key and
  reference-reply strata;
- reference prompt/reply exact overlap and reply 4-/8-gram novelty;
- value-masked prompt and reply template overlap at K = 256 and 1,024;
- an independent `CpuWriterCore` comparison against the integer serving writer;
- explicit `unavailable` entries for data or controls not supplied.

The input UORT, tokenizer hash, manifest hashes, vocabulary, BOS, conversation count and response
inventory are checked before scoring. Empty corpora, zero response runs, `max-targets=0`, malformed
artifacts, unknown or duplicate arguments and an existing output path are refused. An output is
created with `create_new` and never replaced.

The template census follows `train-in-space --census`: TRAIN's K most frequent tokens are kept, all
single-digit tokenizer pieces are masked, every other token is replaced by one mask sentinel, and
adjacent sentinels collapse. A panel is reported as holding new prompt templates when its K = 1,024
masked-prompt share is at most 0.5.

### Parity boundary

For the current 19-table M1 artifact, the independent CPU writer reproduces the serving writer from
the serving trunk's Q12 query: admitted cells, top-8 and top-1. Full two-sided CPU-trunk parity is
reported unavailable because `uor-r4-cpu-train` currently implements the four-table discrete read
path, not the 15 additional M1 trunk tables. This is a writer check, not an alias for a second
independent trunk evaluation.

### Evidence still unavailable without more inputs

- Generated complete-reply correctness and loops need a model-generated reply file.
- Generated-reply novelty needs those generated replies.
- Count-table, memory-off and no-read comparisons need corresponding exported arms.

The public smoke result is recorded in the
[deepseek-1 report](../../loop/tasks/one-model-end-to-end/deepseek-1/REPORT.md).
