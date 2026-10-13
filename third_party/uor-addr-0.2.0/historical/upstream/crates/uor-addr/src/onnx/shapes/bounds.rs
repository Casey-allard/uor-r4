//! ONNX spec-pinned constants.
//!
//! ADR-060 removed the fixed-width two-level commitment
//! (`ONNX_CANON_MAX_BYTES` / `ONNX_CANON_BYTES`) and the
//! application-policy capacity profile (`OnnxHostBounds` /
//! `OnnxAddrBounds`) with its node-count / initializer-count /
//! attribute-count / IO-count / tensor-data ceilings. The realization now
//! emits the **full flat canonical skeleton** (the `ModelProto` structure
//! emitted inline, with variable-length leaves — tensor data, strings,
//! opaque sub-message payloads — replaced by their SHA-256 digests) as an
//! unbounded `alloc` buffer that flows through the pipeline as a borrowed
//! carrier. Every count and width is unbounded.
//!
//! What remains are ONNX **spec / policy constants** (the admitted IR
//! version range, the default-domain opset-version minimum) plus one
//! native-stack-overflow guard on the recursive subgraph descent.

/// The highest ONNX IR version this realization admits — `onnx.proto`'s
/// current `Version::IR_VERSION` (`= 13`, 2026). The realization accepts
/// any `ir_version` in `1..=ONNX_IR_VERSION_MAX`: the canonical skeleton
/// is IR-version-agnostic (the field numbers it reads are stable across
/// IR revisions; IR-v10+ `NodeProto.overload` simply reads empty on older
/// models), and the `ir_version` value itself is bound into the skeleton,
/// so two IR revisions of the same logical model canonicalize distinctly.
/// Admitting the range lets the realization content-address real-world
/// models (published exports are predominantly IR 6–10) rather than only
/// the latest revision.
pub const ONNX_IR_VERSION_MAX: i64 = 13;

/// Policy: the minimum opset version accepted for the default domain `""`.
/// ONNX mandates no minimum (`= 1` accepts any opset); raise per
/// application policy. Inlined from the pre-ADR-060 `OnnxAddrBounds`
/// profile (`ONNX_OPSET_VERSION_MIN = 1`).
pub const ONNX_OPSET_VERSION_MIN: i64 = 1;

/// Native-stack-overflow guard on the recursive subgraph descent
/// (`If` / `Loop` / `Scan` bodies via `GRAPH` / `GRAPHS` attributes).
/// Guards the call stack against pathologically-nested subgraphs; it is
/// not a ceiling on node / attribute count at any level.
pub const ONNX_SUBGRAPH_DEPTH_MAX: usize = 64;
