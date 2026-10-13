# UOR Candle NN 0.9.2 compiled boundary

Owner D29/D30, 13 October 2026: prohibited normalized exponentials, squashing,
dense contraction and CPU model fallback are excluded from the compiled vendor
implementation. This patch introduces no replacement model mechanism.

The complete upstream `src/`, Cargo manifests and their original source checksums
are retained under `historical/`. Original examples, integration tests and benches
are moved there and removed from active target registration. That retained source
is inactive evidence, not an execution permission. The original library API names,
signatures, structs and serialized activation/configuration enum variants remain.

Active changes:

- Remove the private Sigmoid, SoftmaxLastDim and Sdpa custom-op implementations
  entirely, including CPU, CUDA, Metal and backward math. Public normalized,
  squash, attention and probability-loss/sampling entries return recoverable
  Candle errors before reading tensor values or launching a kernel.
- Replace CPU flash-attention and MOE modules with API-compatible Result refusals;
  no affinity, host reductions, matrix kernels, GPU FFI or runtime compilation is
  retained in those active modules.
- Dense Linear/Conv forward and LSTM/GRU step implementations refuse; constructors,
  state/configuration shapes and historical artifact type identities remain.
- Preserve legal GPU normalization implementations, but CPU custom normalization
  and all rotary APIs return Result errors. Vendor RoPE is unused by the qualified
  rung0/Starling path and is excluded from the compiled module. Slow norm wrappers and public
  activation/dropout wrappers refuse non-CUDA tensors before arithmetic.
- Preserve legal GPU ReLU/clamp/leaky/PReLU operations; every historical squash
  activation variant refuses. Preserve optimizer, variable builder/map and
  initialization source byte-for-byte. Core/backend CPU arithmetic admission is
  separately owned; storage/metadata/device transfer remain usable.
- Disable upstream doctests and historical numerical target registration. Add
  one fixed-storage admission/refusal test; its operand storage is metadata only,
  and every tested function must refuse before model arithmetic.
- Keep metal/accelerate/mkl/cudnn feature names as inert compatibility flags,
  remove their backend/vendor dependency edges, and explicitly refuse their
  selection with a compile error. Only the reviewed CUDA feature activates a
  model backend; selecting an inactive name cannot start vendor compilation.

`historical/apply-boundary.rs` is a one-time std-only text-edit recipe, not part of
the compiled module/build tree. Its first narrow marker matched the RNN trait as
well as the two implementations and stopped before writing that file; the corrected
marker targets the two `in_state` method definitions. This was edit-tooling failure,
not model execution or a mechanism result. No source or measurement was discarded.

Validation at authoring: Rust formatting/syntax checks and active-source census.
Production crate compilation and finite CUDA qualification are PENDING and owned
by the integration controller; no optimizer/training/CPU model reference runs here.
The root Cargo patch/lock/source freeze and protected delivery are separate tasks.
