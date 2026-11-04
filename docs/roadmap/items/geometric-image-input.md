---
id: geometric-image-input
title: Decompose images into bounded integer R4 patches for native Studio input
milestone: M7
area: serving
status: done
open_to: lab
needs_gpu: false
depends_on: [studio-conversational-composer]
done_class: host
notebook: docs/research/notebook/m4-serving.md
scope: [Cargo.toml, Cargo.lock, src-tauri/Cargo.lock, xtask/src/main.rs, xtask/src/roadmap_sync.rs, crates/uor-r4-vision/**, crates/uor-r4-integer/Cargo.toml, crates/uor-r4-integer/src/openai.rs, crates/uor-r4-integer/src/loopback.rs, crates/uor-r4-integer/tests/openai_loopback.rs, web/studio/index.html, web/studio/index.css, web/studio/assets/js/**, web/studio/src/**, docs/products/web-client.md, docs/start/FILEMAP.md, docs/research/notebook/m4-serving.md, docs/loop/tasks/geometric-image-input/**, docs/roadmap/items/geometric-image-input.md]
---

## Why it matters

The owner wants the live Studio composer to accept pasted, dropped and selected
pictures as well as text. No image decomposition or trained
visual-understanding artifact exists. A deterministic integer R4 representation
is required before any image can be sent to native `uor-chat`; a captioner,
placeholder success or visual-comprehension claim would be false.

Issue [#3619](https://github.com/UOR-Foundation/uor-r4/issues/3619) carries the
original registration and work card. This item restores that missing roadmap
source to `main`.

## What is missing

All bounded image-input stages are delivered. Text attachments and native
OpenAI streaming remain unchanged; ordinary native `image_url` and image
generation/edit families still return truthful 501 responses.

## Delivered child: integer decomposition primitive

[deepseek-1](../../loop/tasks/geometric-image-input/deepseek-1/REPORT.md)
delivered `crates/uor-r4-vision`: exact bounded RGBA validation, non-empty
positioned patch coverage, Q12 mean RGB/luma/chroma/edges, two signed R4
vectors, source-bound canonical bytes and ordered Rayon evaluation. On the
256 by 256, 32 by 32-grid fixture, 16 workers were 2.81 times as fast as one
over 10,000 decompositions with an identical canonical digest. This is component
evidence only; the native endpoint and validated `uor_r4_image` context were
delivered in the following child.

## Delivered child: native image ingestion

[deepseek-2](../../loop/tasks/geometric-image-input/deepseek-2/REPORT.md)
delivered authenticated `POST /v1/images/decompose` over bounded
`application/octet-stream` RGBA, exact canonical parsing and validation, and
the native `uor_r4_image` content part. Its positioned signed Q12 R4 vectors
enter the ordinary prompt and context preflight; the representation can
therefore consume context or produce a truthful 413, but it is not a trained
visual reader. Ordinary `image_url` and image generation/edit families remain
501. At that stage the Studio Canvas and issue #3619 remained open.

## Delivered child: Studio Canvas input

[deepseek-3](../../loop/tasks/geometric-image-input/deepseek-3/REPORT.md)
delivered browser decode/downsample, memory-only previews, paste/drop/picker
selection, removal and reload reattachment handling. Native Send decomposes the
bounded image after the explicit action and sends a canonical `uor_r4_image`;
custom providers receive an ordinary `image_url`. A real headless-Chromium run
against native `uor-chat` produced exact `34/6/40` prompt/completion/total usage
and displayed the required experimental label. This is input plumbing, not
visual understanding.

## Follow-up hardening: issued R4 records

[kimi-1](../../loop/tasks/geometric-image-input/kimi-1/REPORT.md) binds native image parts to at most 64 server-issued decompositions. Exact eight-field parts require the issued ID, canonical bytes, source Blake3, dimensions and grid; unknown, expired, substituted and non-user parts are refused even with recomputed hashes. Compact base64 preserves every signed Q12 value without the earlier 206/192 context overflow. At `4780a06ba`, real dual-c131 image/text turns returned `I.`: 53/3/56 at 165.0 tok/s and 34/3/37 at 165.4 tok/s after reload; a true 2x2 image previously used 118/3/121. Reload preserves text-only sends and a working Clear path while image bytes remain absent. Native and WASM audits at `81d9ed6ca` pass 44/138 and 7/7 respectively. This remains transport evidence, not visual understanding.

## Done when

- `decompose_rgba(width, height, rgba, grid)` validates exact bounds and produces
  deterministic positioned patches with mean RGB, luma, two chroma axes,
  horizontal and vertical edges, and signed Q12 R4 features. Hand-computed
  fixtures, canonical bytes, source-hash binding, controlled pixel changes and
  failure cases pass using integer-only math.
- `POST /v1/images/decompose` accepts bounded `application/octet-stream` RGBA
  with explicit dimensions/grid and returns the canonical decomposition. Native
  `uor_r4_image` parts validate schema/hash/canonical form and pass normal
  context preflight. Ordinary `image_url` and unsupported OpenAI endpoint
  families retain truthful 501 responses; bounded resource/context refusals
  retain truthful 413 responses.
- Browser Canvas decodes and downsamples images to at most 256 by 256.
  Paste/drop/picker provides previews, names/sizes, removal and the label
  `R4 patch input — experimental; not visual understanding`. Native Send uses
  only successful decompositions after explicit Send; custom providers may use
  ordinary `image_url`; image blobs are memory-only and reload asks for
  reattachment.
- Vision, native integer/OpenAI, Studio, real-browser flow with exact usage,
  native/WASM audit,
  repository gates, fresh exact-head review, green GitHub checks, protected
  queue and fresh-main verification pass. No visual-understanding, useful-chat,
  production, M4/M7 completion or trained multimodal capability is claimed.

## How to start

Create a lab ORDER under `docs/loop/tasks/geometric-image-input/`. Write
hand-computed failing tests before the new internal `uor-r4-vision` crate, then
add bounded native ingestion and plain-JavaScript Canvas integration without new
external runtime dependencies or changes to model, tokenizer or training
semantics.

## Rules that apply

- Integer-only decomposition; no float, dense matmul/GEMM, softmax/squashing,
  learned captioner, external vision model, semantic label or
  visual-understanding claim.
- Preserve the native artifact, existing text/WASM/custom-provider paths,
  loopback admission/token rules, exact request bounds and #3500/#3569
  acceptance tracks.
- No production publish, paid provider, committed image corpus/binary fixture,
  persisted image blob, silent truncation or unsupported image success.

## Cancel if

A deterministic integer decomposition cannot bind dimensions, pixels, patch
geometry and canonical bytes without weakening the existing native image
failure or the no-visual-understanding boundary.

## Sources

- The owner's conversational Studio request in `PLAN.md` and the approved
  Task 4 image-decomposition plan.
- [Studio conversational composer](../../loop/tasks/studio-conversational-composer/kimi-1/REPORT.md)
  and native service #3606 / #3595.
- `crates/uor-r4-integer/src/openai.rs`, `loopback.rs`,
  `web/studio/assets/js/composer_input.js` and `studio_api.js`.
