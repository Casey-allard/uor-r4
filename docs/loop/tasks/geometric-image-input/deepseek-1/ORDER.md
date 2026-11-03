# ORDER: geometric-image-input, deepseek-1 (deterministic parallel RGBA-to-R4 decomposition)

Lab: deepseek. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619). Base: `ab8fc5b0c51d7b160e0520e738c85475ab662853`.

## Goal

Deliver the first bounded child of #3619: an internal `uor-r4-vision` crate
whose `decompose_rgba(width, height, rgba, grid)` primitive validates one
exact RGBA source, partitions it into non-empty positioned patches, computes
integer Q12 visual statistics, binds the source hash and emits canonical bytes.
Patch evaluation must use all available CPU workers without changing output
order or bytes. This unit does not add the native endpoint or Studio UI; #3619
remains open.

## Design

- Accept at most 256 by 256 RGBA pixels and a non-empty grid no larger than the
  source. Partition every axis exactly, so every pixel belongs to one patch.
- Per patch, compute rounded Q12 mean R/G/B, integer Rec.601 luma, the signed
  axes `R-luma` and `B-luma`, and mean signed horizontal/vertical luma edges.
- Expose two signed Q12 R4 vectors: centered RGB plus centered luma, then the
  two chroma axes plus the two edges. Every value fits `i16`; no float,
  surrogate, saturation or model path is used.
- Evaluate patches independently through Rayon's indexed parallel iterator.
  Collection preserves grid order. A no-default-features build remains serial.
- Canonical bytes have a versioned header containing dimensions, grid, source
  Blake3 digest, patch count and every fixed-width patch field in grid order.

## Tests first

Add the failing public-API tests before the implementation:

1. Hand-computed 2 by 2 red/green/blue/white source with a 1 by 1 and 2 by 2
   grid; pin means, luma, chroma, edges, R4 vectors and exact canonical bytes.
2. A one-pixel controlled RGBA change alters the source hash and canonical bytes
   while only the affected patch statistics move.
3. Wrong byte length, zero/oversize dimensions, empty/oversize grid, grid larger
   than the source and arithmetic overflow are distinct errors.
4. Exact partition coverage, input immutability and source-hash recomputation.
5. A deterministic benchmark exercises 256 by 256 at one, sixteen and
   thirty-two Rayon workers and prints one canonical digest.

## Done when

1. The new crate, tests and benchmark implement the design above.
2. Default parallel and no-default-features serial tests pass.
3. Strict Clippy, formatting, public-API documentation and `git diff --check`
   pass.
4. One, sixteen and thirty-two worker runs produce the same canonical digest;
   the retained REPORT records wall time and CPU utilization without claiming
   visual understanding or model capability.
5. The roadmap item records this child as delivered but keeps native ingestion,
   `uor_r4_image`, Canvas input and issue #3619 open.

## Cancel probes

- `rg` finds no `decompose_rgba`, `uor_r4_image`, `/v1/images/decompose` or
  `uor-r4-vision` implementation at the base.
- Rayon 1.10 is already a workspace dependency; no external runtime or new
  image decoder is introduced.
- The item scope already covers the crate, Cargo files, task records, notebook
  and roadmap item. No owner path or shared scope is needed.

## Stop and report

Stop if integer statistics cannot fit the declared Q12 representation, patch
order changes with worker count, canonical bytes fail round-trip binding, or an
image is accepted with incomplete pixels. Do not broaden this unit into native
serving or browser code.
