# ORDER: deterministic integer R4 image decomposition and Studio input

Lab: kimi. Item: [geometric-image-input](../../../../roadmap/items/geometric-image-input.md) (#3619), dependent on delivered [studio-conversational-composer](../../../../roadmap/items/studio-conversational-composer.md) (#3608). Original base `6729e5d507bd39ef400c6515bc29b75f97e682d7`; public base `2789caa9000c28440f881b8ac2b2a4ae7d67edf8`; measurement head `4806f2675c50b799c2c0914f822d7a3e7ad98c81`; earlier review base `35bfa270800e433f4225dbe418d1ecd4be6b28ba`; merged review base `8078271e06138f9e4a185aeb36e3f57f4b068ee0`. The PR is the exact final head/tree identity record.

## Goal

Accept Studio pictures only as bounded integer R4 patches, with native decomposition, validated content parts, Canvas previews and explicit Send. Never claim visual understanding, useful conversation, trained multimodal capability, or production readiness.

## Design

[DESIGN.md](DESIGN.md). One internal vision crate owns decomposition and canonical serialization. The native OpenAI service owns admission, context preflight, and generation. One browser module owns Canvas; existing Studio state remains authoritative.

## Files in scope

- `Cargo.toml`, `Cargo.lock`, `src-tauri/Cargo.lock`, `xtask/src/{main.rs,roadmap_sync.rs}`, new `crates/uor-r4-vision/**`, and narrow `crates/uor-r4-integer/{Cargo.toml,src/openai.rs,src/loopback.rs,tests/openai_loopback.rs}` changes.
- `web/studio/index.html`, `index.css`, `README.md`, `assets/js/**`, and focused `src/adapters/**` tests, including new `image_input.js` and its tests.
- This task folder, `docs/roadmap/items/geometric-image-input.md`, `docs/start/FILEMAP.md`, generated roadmap data when produced by a safe sync, and narrow `docs/products/web-client.md` updates.

## Done when

- Hand-computed 1×1, uniform, asymmetric, odd-dimension, horizontal-edge, vertical-edge, and alpha fixtures prove exact patch rectangles, means, luma/chroma/signs, directional edges, Q12 features, canonical bytes, hash binding, deterministic repeats, controlled pixel changes, and bounded refusals. No float, matmul, semantic caption, or external vision model exists.
- `POST /v1/images/decompose` enforces authenticated loopback, octet-stream, explicit dimensions/grid, exact body length, and 256×256/32-by-32-grid bounds. It returns canonical `R4IMG001` bytes and identity headers, including a server-issued decomposition ID exposed to the browser. Capabilities distinguish decomposition from visual understanding. Ordinary `image_url` and unsupported endpoint families keep explicit 501 responses.
- Native `uor_r4_image` accepts only flat eight-field parts with schema `uor-r4.image-decomposition/1`, matching canonical bytes/source hash/dimensions/grid, and an exact server-issued ID, then uses existing context preflight and exact usage/generation. Unknown, expired, substituted, and non-user image parts are rejected. Overflow returns 413 without truncation.
- Browser Canvas accepts paste/drop/picker images, downsamples proportionally to at most 256×256, releases object URLs, and exposes previews/names/sizes/removal plus `R4 patch input — experimental; not visual understanding`. Native decomposition/send occurs only after successful processing and explicit Send. Custom providers may use ordinary `image_url`; local WASM and unsupported cases fail visibly. Image blobs never persist; reload asks for reattachment.
- Vision/native/Studio suites, previous API/composer checks, real decode→decompose→send browser flows with changed-pixel hashes and exact usage, native/WASM audits, repository gates, fresh exact-head review, green GitHub checks, protected queue, and fresh-main verification pass.
- The PR references #3619/#3628 and leaves #3500/#3569 open. No production publish or M4/M7/useful-chat/visual-understanding/multimodal claim.

## Stop and report conditions

Stop before adding an external runtime dependency, changing model/training/tokenizer/artifact semantics, mounting the typed library, weakening loopback admission or request bounds, silently truncating images/canonical context, persisting image blobs or launch tokens, enabling images without the integer decomposition, adding a captioner/semantic label, publishing production, or editing `.github/**`, another lab's files/issues, or the owner checkout. Merge forward from fresh `main`; never force-push. Reuse the existing Rust/npm runtimes and preserve all unique evidence.
