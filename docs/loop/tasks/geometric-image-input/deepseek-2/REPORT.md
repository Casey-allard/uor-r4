# REPORT: geometric-image-input, deepseek-2

Lab: deepseek. Item:
[geometric-image-input](../../../../roadmap/items/geometric-image-input.md)
(#3619). Order: [ORDER.md](ORDER.md). Base:
`51b159bf96e6abc59ffb40e5e71f5ad365fbe0c7`.

## What did not work, first

- The first endpoint classification returned 413 for a zero width. Zero is a
  malformed dimension and now returns 400; only values above the bound return
  413.
- The first long-body test expected 413, but an exact byte-length mismatch is a
  malformed request. It now returns 400; body lengths above the configured HTTP
  limit still return 413 before decomposition.
- `serde_json::Value` silently collapses duplicate object keys. A recursive
  unique-key deserializer now rejects them before request validation.
- Strict Clippy over `uor-r4-integer` still exits 101 on 51 pre-existing
  diagnostics. Exact base `main` has the same 51, and none names the three
  changed integer files.

## What changed

Added authenticated `POST /v1/images/decompose` with four mandatory, bounded
decimal query parameters. It accepts exact `application/octet-stream` RGBA and
returns canonical `uor-r4-vision` bytes with schema, source digest, dimensions,
grid and patch-count response headers. Bad syntax and exact-length failures are
400; dimension/grid/body resource excess is 413.

Added `parse_canonical` to `uor-r4-vision`. It checks magic, exact length,
dimensions, grid, row-major bounds, Q12 ranges, RGB/luma weighting, chroma axes
and the redundant R4 vectors. The native `uor_r4_image` content part decodes
lowercase canonical hex, reparses it, re-encodes it, checks every repeated
field and source digest, and rejects unknown/missing fields or duplicate JSON
keys.

Accepted image parts render their positioned signed Q12 R4 vectors as
deterministic text. That text is part of the ordinary dialogue encoding and
context check. A single image raises prompt tokens and passes in a larger test
context; the same request returns 413 in the 64-position fixture. Requests are
limited to four image parts. Ordinary `image_url`, `image`, `input_image`, image
generation/edit endpoints and other unsupported families remain 501.

No pixel buffer or image blob is persisted, and no reply or capability behavior
changed outside the supplied prompt context.

## Measurements

- `uor-r4-vision`, default parallel: **9 passed**.
- `uor-r4-vision`, serial: **9 passed**.
- Native loopback: **14 passed**.
- Integer library: **451 passed, 1 ignored**.
- The native loopback tests prove canonical response bytes, digest/geometry
  headers, authentication, bounded 400/413 status, source/schema/hash/geometry
  refusal, duplicate-key refusal, four-image limit, prompt-token growth and
  context preflight.
- No GPU was present and no GPU or visual-understanding result is claimed.

## Checks run at the head

- `rustfmt --check` on every changed Rust file and `git diff --check`: passed.
- `cargo check -p uor-r4-integer --target wasm32-unknown-unknown --features
  serving-wasm --lib`: passed.
- Strict Clippy on `uor-r4-vision` in parallel and serial configurations:
  passed.
- Strict Clippy on `uor-r4-integer` is blocked by the 51 diagnostics reproduced
  unchanged on exact base; the changed integer files have no diagnostic.
- Evidence checksums passed and `uor-harness loop gates --full` passed all 11
  gates on the immediate rerun. The first attempt hit the known fake-gh
  `ETXTBSY` flake in `loop-drive`; both records are retained.

## Skills used

No available Codex skill applied. L-019 was read because the integer source path
matched; this work prepares and validates request context and adds no CPU model
forward or fallback.

## Still open

The Studio Canvas, image preview/removal and memory-only browser blobs remain
open. Native image data reaches a prompt only as experimental untrained R4
context; visual understanding, useful multimodal conversation, issue #3619,
M4 and M7 remain open.

## Plain summary

The native server can now authenticate, validate, canonicalize and carry an R4
image decomposition into model context without hiding it from context limits.
This closes native ingestion but not browser attachment or image understanding.
