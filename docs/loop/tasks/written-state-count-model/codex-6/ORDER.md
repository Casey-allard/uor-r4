# ORDER: value word starts and ordered full connectors through the frozen writer

Lab: codex
Item: [written-state-count-model](../../../../roadmap/items/written-state-count-model.md) (#4063).

The owner asks to keep #4089's matched entity endpoints and exact emitter fixed, isolate a value-word-start eligibility check and a full ordered connector match, and locate the first wrong token relative to the current selector on Wick. #4089 often copies the last piece of “chose” and can stop within a longer connector beginning with “has.” Neither treatment is measured at the current head.

Use a 2×2 comparison and oracle/disabled controls. Reuse identical frozen q/f anchors, four endpoint words, tokenizer classes and connector/value-root state. Represent a connector as an ordered bounded native integer sequence, captured only from the prior TRAIN labels if additional exact token identity is required. Store length and up to three original u32 token IDs losslessly; no float gradients or optimizer. Keep inference prefix-only. Native Rust and AOT CUDA must agree on eligibility, witness and source; measure the actual-feedback writer with independent Rust/SM89 transition replay, digest-bound inputs, prior control equality and independent repeat. Use the two released seed artifacts, same-panel paired divergence and a separate transfer panel. Register treatment, tie, no-match, tokenizer scope and thresholds before empirical work. Preserve negatives. No paid resources or milestone change.
