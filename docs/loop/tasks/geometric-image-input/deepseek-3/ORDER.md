# ORDER: geometric-image-input, deepseek-3 (experimental Studio Canvas input)

Lab: deepseek. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619). Base: `55432f5f88e6c8abd349df88d4944c19bb0c7d23`.

## Goal

Complete the browser-facing experimental image path. Paste, drop and file
picker accept bounded images, Canvas decodes and downsamples each to at most
256 by 256, the composer shows a preview/name/size/removal control, and the
exact label `R4 patch input - experimental; not visual understanding` remains
visible. Native Send decomposes images only after explicit Send and sends exact
`uor_r4_image` parts. Custom providers receive ordinary `image_url` parts.
Image blobs are memory-only.

## Design

- A small `image_input.js` module validates image files, decodes/downsamples
  through Canvas, and returns bounded metadata plus RGBA bytes and a
  downsample preview. Tests may inject the decoder.
- Native image context uses one 1x1 patch by default so an experimental image
  can fit the existing 64-position development stack. The server renders each
  patch's exact signed Q12 R4 bytes compactly in base64; source/dimension/hash
  validation already occurred in the structured content part.
- `studio_api.js` adds the authenticated octet-stream decomposition call and
  accepts structured message content. Custom providers receive the bounded
  preview as `image_url`; native providers receive the canonical
  `uor_r4_image` part.
- The Studio attachment model distinguishes text from image entries. Only text
  attachments use localStorage. Images remain in memory and reload presents a
  reattachment notice through sessionStorage metadata only.
- Local WASM ignores image context with an explicit message. Image-only Send is
  allowed for API providers. No send occurs merely from paste/drop/pick.

## Tests first

Add failing tests for:

1. dimension scaling, exact RGBA length, image MIME/extension, size bounds,
   duplicate replacement and memory-only image metadata;
2. text/image attachment rendering, preview, removal, clear, persistence
   filtering and reattachment notice;
3. native/custom request shaping, canonical hash fields, empty-text image-only
   sends, and errors from decomposition/context preflight;
4. compact server R4 context fitting one 1x1 image in the 64-position fixture;
5. existing composer, streaming, model-source, Studio and worker verifiers.

## Done when

1. The browser modules and Studio integration above pass focused Node/Vitest,
   native OpenAI, WASM and static Studio verification.
2. A real headless Chrome flow loads the live native loopback page, attaches a
   generated image, previews/removes/reattaches it, sends after explicit Send,
   receives exact usage and displays the experimental label.
3. `uor-harness loop gates --full`, exact-head review, protected merge and
   fresh-main verification pass.

## Cancel probes

- The native endpoint and `uor_r4_image` schema are already on `main` at the
  base; no new image codec or model semantics are needed.
- Canvas is available in the Studio's browser and needs no external runtime
  dependency.
- The open Hamming PR does not touch Studio files.

## Stop and report

Stop if decoding would require truncating an image silently, if the image
becomes durable browser storage, if Send happens without an explicit action,
or if native image context bypasses context preflight. Do not claim visual
understanding, useful multimodal chat or M7 completion.
