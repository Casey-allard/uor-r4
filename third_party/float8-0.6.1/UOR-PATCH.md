# UOR CUDA dependency feature boundary

Upstream: float8 0.6.1, copied from the Cargo registry with its MIT license.
Cargo.toml.upstream preserves the original author manifest. The active Cargo
manifests remove cudarc's cublas, cublaslt and nvrtc features. CUDA DeviceRepr
implementations and numerical type definitions are unchanged; they are needed
for Candle's float8 0.6 types (cudarc's separate float8 0.7 types are not a
substitute). This removes transitive activation of forbidden vendor/compiler
features; it is not evidence that any model uses FP8 or that a CUDA binary has
passed qualification.
