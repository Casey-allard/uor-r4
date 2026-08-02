# Observation-first transformerless compilation

The production compiler boundary is the completed recorded corpus, not a
Hugging Face model directory.

There are two separate offline workflows:

1. Capture observations, if new teacher data is required. `observe` and the
   legacy `compile --source` path may load a CPU teacher; that is capture-only
   work and may use matrix operations internally.
2. Compile recorded observations. `compile-recorded` consumes only
   `corpus.meta` and `corpus.records` plus an explicit vocabulary size. It does
   not construct a teacher, load model weights, call the model-source forward
   path, or use GPU backends. The representation is a deterministic signed
   hash projection of the recorded top-k next-token distributions. Calibration
   and store construction use ordered worker reductions, so output bytes do
   not depend on worker count.

The root command is:

```text
cargo run --release --offline --bin r4 -- compile \
  --corpus-meta <completed corpus.meta> \
  --corpus-recs <completed corpus.records> \
  --vocab-size <tokenizer vocabulary size> \
  --output <compiled directory>
```

The corpus metadata must have its completion byte set. A resumable capture
that was interrupted is intentionally rejected until it is complete; this
prevents a partial teacher sample from being mistaken for a finished model.

This path is an observation-derived native model, not a claim of exact
teacher parity. Run the existing held-out evaluation and quality gates before
adopting its artifact.
