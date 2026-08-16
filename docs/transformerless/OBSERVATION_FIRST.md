# Observation-first transformerless compilation

The production compiler boundary is the completed recorded corpus, not a
Hugging Face model directory.

There are two separate offline workflows:

1. Capture observations, if new teacher data is required. `observe` and the
   legacy `compile --source` path may load a CPU teacher; that is capture-only
   work and may use matrix operations internally.
2. Compile recorded observations. The command's data inputs are
   `corpus.meta` and `corpus.records` plus an explicit vocabulary size; it also
   validates their sibling attention+dense execution records when present. It does
   not construct a teacher, load model weights, call the model-source forward
   path, or use GPU backends. The representation is a deterministic signed
   hash projection of the recorded top-k next-token distributions. Calibration
   and store construction use ordered worker reductions, so output bytes do
   not depend on worker count.

`compile-recorded` resolves one source-execution snapshot from the recorded
corpus directory: explicit `attention_operator.json`, optional
`dense_operator.json`, and the sibling observation `manifest.json` are
registry-validated, must agree when multiple records are present, and are
propagated into the compiled output. The resolver snapshots and rechecks the
manifest plus both sidecars jointly so disappearance or cross-era mixing fails
before output mutation. Genuine historical attention absence retains the
implicit `standard-source-attention/1` interpretation; dense absence remains a
typed compatibility state for Llama and pre-#704 corpora.
The command still does not accept a runtime tokenizer as input, propagate a
registered tokenizer identity, or emit `tokenizer.bin`. It must therefore not
be treated as a self-contained text-serving bundle or used to infer/relabel the
recorded token-id space. Carrying tokenizer provenance through this boundary
requires a separate explicit input contract; issue #718 only wires
source-tokenizer consumers that can resolve the original definition.

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
