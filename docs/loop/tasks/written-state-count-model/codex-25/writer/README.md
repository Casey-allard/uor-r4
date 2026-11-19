# D25 writer and client replay

Build from this directory with `cargo build --release --locked`. Use the grade
file, trace, results and learned bank from the same sealed run. Capture one
ordinary process per panel row and artifact with `uor-chat --discrete ARTIFACT
--tokenizer TOKENIZER --protocol 2 --temperature 0 --max-tokens 16 --read-mode
enabled --say TEXT`. Save the JSON as `ORDINARY_DIR/ARM/ROW.json`, where
`ARM` is `ref`, `nl`, `wl`, `nc` or `wc`, and `ROW` is the numeric
panel row from the ordinary prompt file. Only D25 carriers accept the optional
`--contextual-source-trace` flag.

```sh
d25-contextual-source-writer GRADE GPU_TRACE GPU_RESULTS GPU_BANK TOKENIZER \
  ORDINARY_DIR REF_D11 NL_D11 WL_D11 NC_D11 WC_D11 > writer-grade.json
```

The grader validates the frozen parent, tokenizer, common bank, arm identity and
common TRAIN digest. Compare `.bindings.train_sha256` in the emitted receipt
with the independently sealed TRAIN packet digest. The grader compares every
device distance, margin, eligibility bit and pick against the loaded Rust
carrier, including distance receipts for masked rows. The first mismatch
aborts grading.

Ordinary rows use the client's own role and binder. Supplied-query rows keep
the same binder and iterate every fact for the supplied query span.
Supplied-fact-and-query rows observe only the designated assertion, keeping the
selected original onset and running the exact writer from there. The forced
legal controls replay both original onsets. The disabled control follows
ordinary D11 prediction with the writer inactive. Each replay stops at EOS or
after sixteen tokens. Unsupported rows distinguish a factual copy, no source,
and the exact decoded refusal through EOS. The output retains verdicts by row,
writer words, source locations, and the first differing expected and emitted
token IDs, including EOS and early termination.
