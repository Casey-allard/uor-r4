# Runtime-only corpus refresh

The `transformerless runtime-corpus` command refreshes or extends a corpus
using an already compiled table-native artifact and TLS1 store:

```bash
cargo run --release --offline --bin r4 -- transformerless runtime-corpus \
  --artifacts path/to/tless_artifacts.bin \
  --store path/to/tless_store.bin \
  --seed-meta path/to/corpus.meta \
  --seed-recs path/to/corpus.records \
  --out path/to/runtime-corpus \
  --target 500000 \
  --threads 8
```

This path does not load a Hugging Face checkpoint, tokenizer, teacher oracle,
hidden-state buffer, floating-point model, or matrix-multiplication routine.
It relabels the seed stream with the frozen integer runtime, extends the stream
with bounded greedy runtime predictions, and rebuilds the store with ordered,
coarse-grained workers. The output includes copied artifacts, a rebuilt store,
the corpus files, and `runtime_corpus_manifest.json` with byte-level provenance.

The result is deliberately named `runtime-self-distilled-v1`: it is a
transformer-free refresh of the frozen artifact's behavior, not a claim of
parity with the original teacher. Teacher-fidelity numbers must not be mixed
with this corpus unless the evaluation report records that provenance change.

The generated records are deterministic for identical inputs and thread
counts. Store code derivation is parallelized, while evidence insertion remains
ordered so worker scheduling cannot change the output bytes.
