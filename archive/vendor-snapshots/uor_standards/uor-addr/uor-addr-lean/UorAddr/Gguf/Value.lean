/-!
# GGUF typed input + canonical commitment byte form.

ADR-060: the realization reduces a GGUF v3 file to the **full flat
Merkle skeleton** (see `crates/uor-addr/src/gguf/value.rs`): a header,
then metadata KVs sorted by key, then tensor info sorted by name, with
every variable-length leaf (string, array payload, tensor data) replaced
by its streamed SHA-256 digest. There is no two-level commitment and no
count / width ceiling. Modelled here as the canonical byte sequence.
-/
namespace UorAddr.Gguf

/-- The canonical flat skeleton the ψ-pipeline hashes — a byte sequence. -/
abbrev Commitment := List UInt8

/-- A 32-byte streamed leaf digest (a string / array payload / tensor's
data region inlined into the skeleton). -/
abbrev LeafDigest := Fin 32 → UInt8

end UorAddr.Gguf
