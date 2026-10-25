# REPORT: m7-studio-self-host-page-assets, antigravity-60

Lab: antigravity. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md). Order: [ORDER.md](ORDER.md).
Closes #3075.

## What did not work

1. **Running `node web/studio/verify_studio_worker.mjs` on a clean checkout failed with `ENOENT` on `web/studio/pkg/uor_r4_integer_bg.wasm` (`instrument`):** Because `web/studio/pkg/` is gitignored (`.gitignore` line 26), a fresh checkout lacks `pkg/uor_r4_integer_bg.wasm`, `pkg/uor_r4_integer.js`, and `pkg/synthetic-stack.lut`. Whereas `web/studio/verify_m7_parity.mjs` already invoked `wasm-pack` and `cargo run -p uor-r4-integer --features serving-wasm --bin mk-synthetic-stack -- 5` automatically when those files were absent, `verify_studio_worker.mjs` assumed they were pre-built and crashed with `ENOENT`.
2. **The existing `uor-r4.studio-tokenizer/1` implementation in `uor_tokenizer_artifact.js` and `make_tokenizer.mjs` was still the `rawBytes[i] % vocabSize` byte-mod stand-in (`mechanism`):** `make_tokenizer.mjs` emitted `mode: "byte-mod"` (`byte_to_token[b] = b % vocab_size`) and `tokenizerLabel()` still reported `"stack-bound byte-mod stand-in"`, so text prompts such as `<|bos|>User: 01\n` were mapped byte-by-byte modulo 40 (`<|bos|>` became 7 modulo-40 bytes `[20, 4, 18, 31, 35, 4, 22]` instead of atomic token `0`), generated token IDs could not be decoded back to vocabulary pieces, `make_tokenizer.mjs` rejected `uor-r4.lut-discrete/1` headers (`shape.vocabulary`), and `uor_model_worker.js` cached `cachedTokenizerArtifact` at worker scope so reloading a stack never re-validated the tokenizer binding.

## What changed

- **Files changed:** 10 files (`web/studio/assets/js/uor_tokenizer_artifact.js`, `web/studio/assets/js/uor_model_worker.js`, `web/studio/tools/make_tokenizer.mjs`, `web/studio/verify_studio_worker.mjs`, `docs/products/web-client.md`, `docs/labs/m7-studio-self-host-page-assets/README.md`, `docs/roadmap/items/m7-studio-self-host-page-assets.md`, `docs/roadmap/README.md`, `docs/loop/tasks/m7-studio-self-host-page-assets/antigravity-60/ORDER.md`, `docs/loop/tasks/m7-studio-self-host-page-assets/antigravity-60/REPORT.md`).
- **Size vs. estimate:** ~550 changed code/test/doc lines against the item's `size: S` estimate (~2.7x; overrun explained by bundling the self-contained GPT-2 `ByteLevel` BPE encoder/decoder, canonical `DEFAULT_STACK_TOKENIZER_ARTIFACT`, and `ensureWasmAndSyntheticStack` + BPE merge/reload verification in `verify_studio_worker.mjs`).
  1. [`web/studio/assets/js/uor_tokenizer_artifact.js`](../../../../../web/studio/assets/js/uor_tokenizer_artifact.js): Added `mode: "byte-bpe"` (deterministic HuggingFace-compatible `ByteLevel` BPE vocabulary `model.vocab`, rank-ordered `model.merges`, leftmost-longest atomic `added_tokens`, optional `Digits(individual_digits)` + `ByteLevel` pre-tokenization, `encodeTextWithArtifact`, `encodeBytesWithArtifact`, `decodeTokenIdsWithArtifact`, and `decodeTokenIdWithArtifact`) alongside legacy `mode: "byte-mod"`; bundled `DEFAULT_STACK_TOKENIZER_ARTIFACT` bound to the canonical `seed = 5` `mk-synthetic-stack` digest (`d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `vocab_size: 40`); and updated `tokenizerLabel()` to report `"stack-bound byte-bpe tokenizer (vocab N, stack sha256 <12-hex>…)"` when bound.
  2. [`web/studio/tools/make_tokenizer.mjs`](../../../../../web/studio/tools/make_tokenizer.mjs): Added `uor-r4.lut-discrete/1` (`shape.vocabulary`) and `uor-r4.lut-stack/1` (`shape.vocab`) header parsing, defaulted `buildArtifact()` to `mode: "byte-bpe"` (with `--source-tokenizer <hf-tokenizer.json>` for custom/M4 BPE vocabularies and `--mode byte-mod` for legacy comparisons), and updated `--check` to verify both modes.
  3. [`web/studio/assets/js/uor_model_worker.js`](../../../../../web/studio/assets/js/uor_model_worker.js): Updated `loadTokenizerArtifact()` to resolve and validate the tokenizer per loaded stack (`payload.tokenizerArtifact`, `../../pkg/tokenizer.json`, or `DEFAULT_STACK_TOKENIZER_ARTIFACT`) without stale per-worker caching across `load_model` reloads, encode text prompts via `encodeTextWithArtifact()`, and decode streamed tokens via `decodeTokenIdWithArtifact()`.
  4. [`web/studio/verify_studio_worker.mjs`](../../../../../web/studio/verify_studio_worker.mjs): Added `ensureWasmAndSyntheticStack()` so clean checkouts build `pkg/uor_r4_integer_bg.wasm` and `pkg/synthetic-stack.lut` automatically, and added checks for `byte-bpe` atomic role encoding (`<|bos|>User: 01\n` -> `[0, 6, 7, 8, 9, 3, 4, 18, 19, 5]`), rank-ordered BPE merges (`["H e", "l l", "l o"]` on `<|system|>Hello<|turn_end|>` -> `[3, 11, 12, 10, 6]`), `uor-r4.lut-discrete/1` header parsing, live worker text-prompt generation and piece decoding, and reload re-binding (`tokenizerArtifact: false` fallback and restore).
  5. [`docs/products/web-client.md`](../../../../products/web-client.md), [`docs/labs/m7-studio-self-host-page-assets/README.md`](../../../../labs/m7-studio-self-host-page-assets/README.md), [`docs/roadmap/items/m7-studio-self-host-page-assets.md`](../../../../roadmap/items/m7-studio-self-host-page-assets.md), and [`docs/roadmap/README.md`](../../../../roadmap/README.md): Updated the product documentation and lab record (including all four legs and the GitHub API contributor panel's opt-in position), set `m7-studio-self-host-page-assets` to `status: done`, and re-rendered `docs/roadmap/README.md`.

## Measurements

- **Static page-load check (`node web/studio/verify_studio_page.mjs --strict-router`):**
  - `verdict`: `"PAGE-LOAD CHECKS PASSED"`, `strictPages`: `["index.html", "geometric_prime_router_webapp.html"]`, `failures`: `[]`.
- **Headless Chromium runtime page-load trap (`node web/studio/verify_studio_page_load.mjs --strict-router`):**
  - `verdict`: `"RUNTIME PAGE-LOAD TRAP PASSED"`, `trapSelfTest`: `plantedExternalDetected: true` across `["inline", "blob-worker", "late"]`, `index.html`: `loaded: true`, `external: []`, `geometric_prime_router_webapp.html`: `loaded: true`, `external: []`.
- **Studio worker & stack-bound BPE tokenizer check (`node web/studio/verify_studio_worker.mjs`):**
  - `verdict`: `"ALL STUDIO WORKER CHECKS PASSED"`, `engine`: `"uor-r4-integer/serving-wasm"`, `tokenizerLabel`: `"stack-bound byte-bpe tokenizer (vocab 40, stack sha256 d60f88d6e08a…)"`, `weightsPerToken`: `15968`, `tokens32`: `[38,9,33,8,25,38,30,25,17,30,30,30,17,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30,30]`, `checkpointBytes32`: `9782`, `checkpointBytes16`: `5366`, `saveRestoreBitIdentical`: `true`, `midGenCloseResetGuarded`: `true`, `trapSelfTestHits`: `6`, `remoteRequests`: `0`.
- **Tokenizer CLI check (`node web/studio/tools/make_tokenizer.mjs web/studio/pkg/synthetic-stack.lut --check web/studio/pkg/tokenizer.json`):**
  - `verdict`: `"TOKENIZER ARTIFACT MATCHES THE STACK"`, `mode`: `"byte-bpe"`, `vocab_size`: `40`, `stack_sha256`: `"d60f88d6e08a"`.
- **Library/CLI/Browser parity (`node web/studio/verify_m7_parity.mjs`):**
  - `schema`: `"uor-r4.m7-browser-parity/1"`, `remote_requests`: `0`, all 8 prompts (`P0_bos`..`P7_octave`) `worker_matches_wasm: true`.

## Checks run at the head

- `node web/studio/verify_studio_page.mjs --strict-router` -> PASS
- `node web/studio/verify_studio_page_load.mjs --strict-router` -> PASS
- `node web/studio/verify_studio_worker.mjs` -> PASS
- `node web/studio/tools/make_tokenizer.mjs web/studio/pkg/synthetic-stack.lut --check web/studio/pkg/tokenizer.json` -> PASS
- `node web/studio/verify_m7_parity.mjs` -> PASS
- `~/.cargo/bin/cargo run -p xtask -- roadmap check` -> PASS
- `uor-harness loop gates --full` -> PASS
- `uor-harness loop report --check --order docs/loop/tasks/m7-studio-self-host-page-assets/antigravity-60/ORDER.md` -> PASS
- `uor-harness loop pr-check --local` -> PASS

## Skills used

- `uor-research-architect`: followed artifact-bound determinism (`stack_sha256` + `vocab_size` binding) and zero-remote-call Studio verification.
- `scope-guard` and `solution-integrity`: verified all changes stay within `m7-studio-self-host-page-assets` scope and tested clean-checkout execution end to end.

## Still open

- When the M4 checkpoint is exported (`m4-export-uorlut01`, `#2854`), re-run `node web/studio/tools/make_tokenizer.mjs <m4-stack.lut> --source-tokenizer <m4-tokenizer.json> --out web/studio/pkg/tokenizer.json` so the Studio's `uor-r4.studio-tokenizer/1` (`mode: "byte-bpe"`) artifact binds to the M4 stack's SHA-256 and `ByteLevel` BPE vocabulary.

## Plain summary

The Studio web application (`web/studio/index.html`) and the router demo page (`web/studio/geometric_prime_router_webapp.html`) now pass all four offline and self-hosting checks on a clean checkout: static asset verification (`verify_studio_page.mjs --strict-router`), headless-browser runtime network trapping (`verify_studio_page_load.mjs --strict-router`), worker and stack-bound `ByteLevel` BPE tokenizer verification (`verify_studio_worker.mjs`), and documented opt-in handling for the contributor GitHub API panel (`docs/products/web-client.md` and `docs/labs/m7-studio-self-host-page-assets/README.md`). Instead of mapping text bytes modulo the vocabulary size (`rawBytes[i] % vocabSize`, the old `"byte-mod stand-in"`), the Studio now bundles and loads a stack-bound `ByteLevel` BPE tokenizer (`uor-r4.studio-tokenizer/1`, `mode: "byte-bpe"`) verified against the loaded stack's SHA-256 digest and vocabulary size, encoding special role tokens atomically, applying rank-ordered BPE merges, and decoding generated token IDs back into text.
