/-!
# ONNX typed input + canonical commitment byte form.

ADR-060: the realization reduces a `ModelProto` to the **full flat
skeleton** emitted inline (`LE_i64(ir_version)`, then opset imports, then
the graph laid out node-by-node in Kahn-topological order with subgraphs
recursed inline, then model metadata), with variable-length leaves
replaced by their streamed SHA-256 digest. There is no two-level
commitment and no count / width ceiling. See
`crates/uor-addr/src/onnx/value.rs`.
-/
namespace UorAddr.Onnx

/-- The canonical flat skeleton the ψ-pipeline hashes — a byte sequence. -/
abbrev Commitment := List UInt8

end UorAddr.Onnx
