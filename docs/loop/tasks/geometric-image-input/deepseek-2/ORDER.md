# ORDER: geometric-image-input, deepseek-2 (bounded native image ingestion)

Lab: deepseek. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619). Base: `51b159bf96e6abc59ffb40e5e71f5ad365fbe0c7`.

## Goal

Make the merged integer R4 decomposition usable through native `uor-chat`
without claiming visual understanding. Add the bounded
`POST /v1/images/decompose` route, exact canonical parsing and validation, and
the `uor_r4_image` native content part. Image-derived R4 values enter the normal
text context deterministically and are covered by the existing token/context
preflight, so they cannot silently disappear or exceed the artifact context.
Studio Canvas work remains the next unit.

## Design

- `POST /v1/images/decompose?width=W&height=H&columns=C&rows=R` accepts exactly
  `application/octet-stream` RGBA bytes and returns the canonical
  `uor-r4-vision` bytes as `application/octet-stream`.
- Query parameters are single, decimal, bounded and mandatory. Wrong lengths
  and malformed grids are 400; bounded resource excess and context overflow are
  413. Authentication, Host and Origin checks remain unchanged.
- `uor_r4_image` carries `schema`, lowercase `canonical_hex`,
  `source_blake3`, width, height and grid. The server decodes and reparses the
  canonical bytes, re-encodes them, and verifies every repeated field and the
  source digest before accepting the part.
- Canonical parsing verifies the magic, exact length, patch count, row-major
  grid positions, partition bounds, field ranges and redundancy between named
  statistics and both R4 vectors.
- Each native image part renders its positioned R4 vectors as deterministic
  ASCII context. The normal dialogue encoder and context feasibility check
  include that rendering. This makes the representation visible to the model
  without claiming it can understand images.
- Ordinary `image_url`, `image`, `input_image` and image generation/edit
  endpoint families remain truthful 501 responses. No image blob is persisted.

## Tests first

Add failing tests before implementation for:

1. canonical parse/round-trip and corruption of magic, length, patch count,
   geometry, vector/stat redundancy, digest and field ranges;
2. endpoint authentication, exact RGBA acceptance, canonical response bytes,
   query validation, body bounds and bounded 400/413 failures;
3. valid `uor_r4_image` context, exact rendered prompt-token increase, malformed
   schema/hash/hex/canonical/dimensions/grid, duplicate fields and image-count
   bounds;
4. an image-derived context that passes preflight at a larger test context and
   returns 413 at the normal small fixture;
5. preservation of ordinary `image_url`, unsupported image families, text,
   SSE and exact usage behavior.

## Done when

1. The endpoint and native content part pass the tests above with bounded
   allocation and no partial success.
2. `uor-r4-vision`, native `uor-r4-integer` serving and focused Clippy/tests,
   formatting, evidence checksums and `uor-harness loop gates --full` pass.
3. The roadmap and notebook record native ingestion as delivered while Canvas,
   issue #3619, visual understanding and model capability remain open.
4. A fresh exact-head review and protected merge verify the change on `main`.

## Cancel probes

- `rg` at the base finds no `/v1/images/decompose`, `parse_canonical` or
  `uor_r4_image`; ordinary image parts still return 501.
- The primitive's canonical fields cover dimensions, grid, source hash and all
  patch statistics, so validation needs no pixels or new format.
- Claude's open #3624 does not edit `openai.rs`, `loopback.rs` or
  `openai_loopback.rs`; this unit can preserve it.
- Rayon, Blake3 and Hex are existing workspace dependencies.

## Stop and report

Stop if native image context cannot be validated deterministically, if adding
it would silently bypass token/context preflight, if the ordinary image failure
weakens, or if another order takes the native serving files. Do not implement
Canvas or claim visual understanding in this unit.
