# uor-r4-graph-format

Packed R4G1 artifact format: types, canonical serializer, two-stage
validation, and borrowed `GraphView` access — the wire foundation of the R⁴
holographic graph compiler.

The authoritative spec is `docs/transformerless/R4G1.md` (wire-format RFC).
This crate implements it as the draft line (`format_version.major = 0`; stable
R4G1 starts at `major >= 1` only after packed widths freeze — see the RFC §8
version gate).

## What it provides

- **Newtypes** (`types.rs`): `NodeId`, `SectionOffset`, `TokenId`, `ScoreQ`
  (Q16.16 carrier), `Depth`, `Radius`, `ArtifactCid`, `SectionId` with the
  section inventory (HEAD/CODE/NODE/EDGE/ROUT/EMIT/EXCT/PROV/CERT/PTCH/SECT)
  and mandatory/optional classification.
- **Stage-1 structural validation** (`header.rs`, `view.rs`): magic, version,
  endianness marker, alignment, `total_len`, sorted non-overlapping
  section table, checked offset arithmetic, unknown-mandatory-section and
  feature-bit rejection, blake3 `artifact_cid`/`head_cid` integrity.
- **Stage-2 semantic validation** (`stage2.rs`, `head.rs`, `records.rs`,
  `rout.rs`): the 224-byte HEAD payload (identities + bounded-work constants
  A/C/W/E/K/D), `PackedNode` range resolution with checked arithmetic, edge
  endpoint bounds, stable edge-kind discriminants (with optional-kind space),
  child/forward index wiring checks, reverse-index coverage, HEAD-bound
  honesty, ROUT v0 bytecode validation (opcodes, operands, forward-only jumps,
  depth ≤ D), EMIT/EXCT storage descriptors, signature word-aligned extents +
  zero-padding checks.
- **`GraphView<'a>`** (`view.rs`): constructed only after both stages pass;
  borrowed zero-copy access to sections, HEAD, nodes, edges, and the reverse
  index. Includes `verify_tokenizer_cid(&self, tokenizer_bytes)` to validate BLAKE3
  tokenizer hashes against `R4G1Header::tokenizer_cid`. Never deserializes into heap object graphs.
- **Canonical serializer** (`ser.rs`): `ArtifactBuilder` — deterministic
  container bytes (identical inputs ⇒ identical bytes), CID computation.
- **Errors** (`error.rs`): one focused `FormatError` variant per invariant;
  no panics on any malformed input.

## Guarantees

- `#![forbid(unsafe_code)]`; no_std feature ladder: `default = ["std"]`,
  `alloc`-only operation supported (`--no-default-features [--features alloc]`).
- Fuzz-hardened: `fuzz/` cargo-fuzz targets (`parse_arbitrary`,
  `mutate_valid`) plus a stable deterministic mutation smoke test —
  malformed bytes always produce structured errors, never panics.
- 60+ tests split across `tests/stage1.rs`, `tests/stage2.rs` (one rejection
  case per invariant) and shared fixture builders in `tests/common/`.

## Downstream contract

Runtime and compiler crates must treat `GraphView` as the only construction
path (Assumption A1 of the proof model): every slice, offset, range,
discriminant, and declared capacity is proven within the artifact bytes before
any runtime table read (Theorem 8).
