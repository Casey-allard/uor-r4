# Rust BPE trainer implementation plan

1. Add public-behavior tests for deterministic merges/reserved IDs/roundtrip and observe RED.
2. Add trainer module using weighted pre-tokens and incremental pair index; share existing pretoken segmentation helper with encoder. Preserve encoder behavior through its existing tests.
3. Add bounded train-bpe CLI with repeated input files, optional pretokenizer JSON, vocab/min-frequency/resource limits and create-new output. Exercise real temporary-file CLI path.
4. Add oracle/edge tests, run focused tokenizer tests/clippy/format. Independent Rust, security and quality review, fix material findings.
5. Canonical record/current-state, protected PR/review/delivery; verify whole delivered tree, preserve unique source before own-only cleanup. Host-only; A8 experiment unchanged.
