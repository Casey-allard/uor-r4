# DESIGN: deterministic integer R4 image input

## Boundary

This unit adds a deterministic representation and transport, not vision. The existing native `IntegerStackModel`/`StackChat`, model artifact, tokenizer, and training semantics do not change. DeepSeek's internal `uor-r4-vision` crate owns decomposition and canonical serialization; the integer engine depends on it, not the reverse. Browser decoding stays in browser-native Canvas. No external runtime dependency or production publication is added.

## Integer decomposition and canonical contract

`decompose_rgba(width, height, rgba, grid)` and `parse_canonical` retain the upstream contract verified by deepseek-1 and deepseek-2: dimensions are bounded to 256, grids to 32 by 32 and to the source dimensions, and every patch has positioned integer RGB/luma/chroma/directional-edge statistics and two signed Q12 R4 vectors. Canonical bytes use the `R4IMG001` magic and a Blake3 source hash. Parsing validates the complete redundant representation before native use. No float, dense matrix multiplication, learned captioner, external vision model, or semantic label exists.

The older `R4IMG1;sha256=...` text prompt, `source_sha256`, `canonical_sha256`, and `canonical_prompt` fields are not part of this contract and must not be reintroduced.

## Native issuance and ingestion

`POST /v1/images/decompose?width=...&height=...&columns=...&rows=...` requires authenticated loopback access and `application/octet-stream` RGBA with exact body length. It returns the canonical bytes plus `X-UOR-Image-Schema`, `X-UOR-Source-Blake3`, dimension/grid, patch-count, and `X-UOR-Decomposition-Id` headers. The ID header is included in CORS exposure.

The serving instance keeps at most 64 issued decompositions in a mutex-protected FIFO registry. Each accepted chat image is a flat content part with exactly eight fields: `type`, `schema`, `canonical_hex`, `source_blake3`, `width`, `height`, `grid`, and `decomposition_id`. Schema is `uor-r4.image-decomposition/1`; grid contains only `columns` and `rows`. The adapter parses and canonicalizes the bytes, verifies all repeated fields, then requires the ID and every content field to match that server's issued record. Unknown, expired, or substituted content fails with 400 even when the replacement is independently valid and its checksums are recomputed. Image parts are accepted only in user turns.

An accepted image uses `render_r4_context` to encode the exact big-endian bytes of its two signed Q12 vectors per patch as compact base64, prefixed by dimensions, grid, and patch count. This lossless representation avoids the 206-position verbose-text overflow on the 192-position native artifact without truncating any feature. The source hash remains validated transport identity rather than prompt text. Existing tokenizer/context preflight and exact usage/generation then apply; four images still overflow the 64-position fixture and return 413. Ordinary `image_url`, image generation/edit, embeddings, and other unsupported families retain explicit 501 responses.

## Browser behavior

`image_input.js` uses object URLs and Canvas to decode a selected/pasted/dropped raster image, downsample proportionally to at most 256×256, extract RGBA, and release resources on every path. It refuses undecodable/oversized input loudly and rejects decoded source dimensions above 4096 pixels per side before canvas allocation or drawing. This is a post-decode guard, not a bound on decoder memory; header parsing before full decode remains future work. The browser independently caps the default grid at 2 columns and 2 rows per axis, so extreme-aspect images can be 1×2 or 2×1 rather than square.

Native preparation posts RGBA with the generation abort signal and `cache: 'no-store'`, checks the signal between images, and reads the binary body only after a successful response. It validates the schema, Blake3 hash, issued ID, exact dimensions/grid, patch count, canonical magic, and byte bound, then converts the canonical bytes to lowercase hex. Error envelopes retain their server message. `apiMessages` emits the exact flat eight-field part only after successful preparation and explicit Send. Custom OpenAI providers may send ordinary data-URL `image_url` parts and surface provider errors verbatim. Local WASM refuses image context explicitly.

Attachments retain one owner in `window.__studioUi`. Image previews and blobs remain in memory; reload displays a reattach request rather than pretending blobs persisted. The visible label is exactly `R4 patch input — experimental; not visual understanding`. Nothing in the UI or docs claims captioning, recognition, question answering, or a trained multimodal artifact.

## Dark theme polish

The owner requested a more Apple-like dark appearance without a layout or behavior change. A token-only refined graphite palette uses `#000000`, `#1C1C1E`, and `#2C2C2E` surfaces, `#F5F5F7` primary text, and a measured `#006EE6` primary/`#64D2FF` bright accent. Existing Apple system typography, controls, targets, radii, and light-theme layout remain unchanged; the light text-accent token is corrected to `#8f3f14` for contrast. This intentionally overrides the generic UI-design prohibition on system fonts because the requested platform is Apple's own visual language. Contrast tests cover all changed text/accent pairs at 4.5:1 or better; no new dependency, gradient, animation, or layout is introduced.

## Verification

The post-merge regressions begin with failing binary transport, flat-part, and missing-issuance tests. Native coverage proves exact route bytes/headers, valid generation/context growth, unknown and expired IDs, valid-content substitution under a reused ID, malformed fields, non-user roles, image count, and context overflow. Browser coverage proves query/body transport, canonical hex conversion, header validation, error envelopes, native/custom provider separation, and reattachment. Run all previous native/Studio checks, native/WASM audits, repository gates, and real built-in-browser decode → decompose → explicit send with exact usage and changed-pixel identities. Deliver through fresh exact-head review, green GitHub checks, protected queue, and fresh-main verification.
