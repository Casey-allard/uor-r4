/-!
# GGUF spec / stack-safety bounds.

Mirrors `crates/uor-addr/src/gguf/shapes/bounds.rs`. ADR-060 removed the
application-policy capacity profile (`GgufHostBounds` and its KV /
tensor / string / array ceilings): the canonical form is the full flat
Merkle skeleton flowing through a borrowed carrier, so every count and
width is unbounded. What remains are GGUF v3 spec constants plus a
native-stack overflow guard on the recursive ARRAY-metadata measurer.
-/
namespace UorAddr.Gguf

/-- `GGUF_VERSION_REQUIRED` — the only admitted version. -/
def versionRequired : Nat := 3

/-- Default tensor-data alignment (`GGUF_DEFAULT_ALIGNMENT`). -/
def defaultAlignment : Nat := 32

/-- Maximum tensor rank (`GGUF_MAX_DIMS` / `GGML_MAX_DIMS`) — a format
constant, not an application cap. -/
def maxDims : Nat := 4

/-- ARRAY-of-ARRAY metadata recursion stack-safety guard
(`GGUF_METADATA_ARRAY_DEPTH_MAX`) — not a content ceiling. -/
def metadataArrayDepthMax : Nat := 64

end UorAddr.Gguf
