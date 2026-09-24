# Offline Rust training foundation

This crate is the shared location for differentiable research references and future training bridges. No serving crate depends on it. Its first executable mode establishes numerical parity and a language-loss gradient through the retained #1017 ordinary transformer. It does not implement a native replacement or a hard-selection relaxation.

## Integrity mode

```text
uor-r4-training integrity SNAPSHOT TOKENS_U16 OUT_JSON {cpu|metal} WINDOW
```

`WINDOW` is 8–64 inputs; the command reads exactly `WINDOW+1` little-endian token IDs from offset zero and hashes the full source store. Use a new `OUT_JSON` file for each attempt. The file is created exclusively after argument validation and before model loading. Completed and failed results are retained. No optimizer step or checkpoint modification occurs. Finite-difference perturbations are temporary and restored.

The retained snapshot is `/Users/casey.allard/uor-r4/.uor-models/research/issue-1017/export`. Its `model.safetensors` SHA-256 is `cf988a6b5f722b614740b0843807a41feff8276571c846bbee85d410a0030d84`. Loading rejects other weights or shapes. All 56 F32 tensors become named differentiable variables. The tied output reuses `model.embed_tokens.weight`.

CPU is the default build. Optional `metal` enables Candle Metal; a requested device never falls back silently. Optional `reference-accelerate` enables the existing Rust oracle's Apple Accelerate comparison path. Builds used for evidence set `UOR_BUILD_SOURCE_COMMIT` to the actual committed source. Candle core and nn are intentionally pinned to **0.9.2**, an established published release; this is not a claim that it is the newest release.

The forward pass composes differentiable RMSNorm, half-rotation HF Llama RoPE, causal softmax, Q/K/V/O projections and SwiGLU. It avoids Candle's inference-only fused softmax/RMSNorm helpers. Every logit is compared; the report includes per-position maximum error and top-one identities, all parameter gradient norms, Q/K/V gradient availability for all six layers, and three finite-difference coordinates chosen as the largest absolute gradient in layer-zero Q, K and V. The finite-difference tolerances and parity ceiling are constants in the source, fixed before execution. Full final-position logits are retained for independent inspection. The reference oracle reports its actual selected arithmetic owner, including an exact-mode environment override if present.

## Scope and next integration

A successful integrity report establishes agreement for that token window and an operative gradient through **soft** attention. It does not establish hard admission, quantization parity, general language ability, a geometric advantage or final serving compliance. The prior #1017 confirmation set has already been opened; it can serve as a historical regression set, never a fresh final draw. Future corpus evaluation/training modes should extend this crate and preserve separate development and final protocols.

The old safetensors/config/tokenizer and Rust reference loader are reused directly. The Python pickle checkpoint and optimizer state are not loaded. A future quantized bridge must explicitly specify its surrogate, gradient path, code usage and hard exported evaluation rather than assuming autodiff differentiates argmax.

Primary implementation references: [Candle 0.9.2 API](https://docs.rs/candle-core/0.9.2/candle_core/), [Candle training example](https://github.com/huggingface/candle/blob/main/candle-examples/examples/mnist-training/main.rs), [tensor operations and inference-only softmax](https://github.com/huggingface/candle/blob/main/candle-nn/src/ops.rs). The actual retained architecture is implemented in `tools/r4-softmax-trainer/src/r4_softmax_trainer/model.py`; this crate reproduces it in Rust as an offline comparator.

Executed September 24: the pinned 32-position check passed on CPU and Metal; see [the compact evidence](../../docs/evidence/reference-autodiff-integrity-2026-09-24.json) and [evaluator manifest](../../docs/integration/reference-evaluator-v1.json). The executable enforces reference weights and architecture and emits full input identities; the complete manifest is verified independently rather than automatically enforced. The baseline table and native student remain future work.
