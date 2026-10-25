# Studio page-load assets and stack-bound tokenizer (`m7-studio-self-host-page-assets`)

Residual item of `m7-studio-connect-wasm-engine` (#2868); see the closure note in #3072. Lab: deepseek (`deepseek-51`, draft PR #3073).

## 1. What is verified now (this unit)

`web/studio/verify_studio_page.mjs` is a **static** scan of every load-time reference in the Studio HTML pages — `script[src]`, `img[src/srcset]`, `link[href]` for loading `rel`s, `iframe/source/video/audio/object` load attributes, and CSS `url(...)` — and asserts each resolves on-origin. Navigation links (`<a href>`) are not load requests and are deliberately excluded (the GitHub links on the page are navigation). The scanner has a fixture self-test that plants four external references (including a protocol-relative script and a `url()` in inline CSS) and proves it reports exactly those while ignoring an `<a href>` navigation link.

At the current head (after deepseek-61):
```text
node web/studio/verify_studio_page.mjs --strict-router
  index.html: clean (zero page-load external references)
  geometric_prime_router_webapp.html: clean (zero page-load external references)
node web/studio/verify_studio_page_load.mjs --strict-router
  index.html: 6 requests, 0 external
  geometric_prime_router_webapp.html: 2 requests, 0 external
```
Both the static scan and the headless-Chrome runtime trap now prove the claim on both Studio pages; the planted-external self-test still proves the traps fire.

**Runtime page-load trap (deepseek-53):** `web/studio/verify_studio_page_load.mjs` loads the pages in headless Chrome through the DevTools protocol (Node's global WebSocket plus a built-in static http server; no npm dependency) and records every network request the page makes while loading. A planted-external self-test proves the trap fires. Result on the current tree: **index.html 6 requests and 0 external; the router demo page 2 requests and 0 external.** Before deepseek-61 the demo page made 9 requests with 8 external URLs (Tailwind plus its build, Font Awesome css + webfont, Google Fonts css + 3 font files) and `--strict-router` exited 1 on it; that legacy residual is gone. The trap auto-attaches to worker targets and waits a settle window, and its self-test plants an inline fetch, a blob-Web-Worker fetch and a deferred fetch to prove all three channels are captured. Chrome is required and the harness fails closed when it cannot launch; the static scan (`verify_studio_page.mjs`) remains the dependency-free companion.

## 2. Demo-page decision — resolved by self-hosting (deepseek-61)

The earlier decision card offered retire/bypass/rewrite because faithful self-hosting was believed to be a bespoke CSS/SVG rewrite. **self-hosting turned out tractable**, so none of those owner/product steps is needed:

- The page's **own inline `tailwind.config`** (the custom `cyber-*` palette) was compiled with the Tailwind v3.4.17 standalone CLI against the page's source, producing `web/studio/assets/css/router_demo_offline.css` (**20,091 B**) containing the utilities used by the page's 263 class tokens (the CSS has 249 unique class selectors). The generated file carries a header naming the build and MIT attribution.
- The **Font Awesome** dependency became local `::before` unicode rules for the 21 icon classes the page uses (no webfont, no external font).
- The **Google Fonts** dependency became the same system-font fallback pattern `uor_offline_fonts.css` already uses for the main page (`'Inter', system-ui...`, `'Fira Code', ui-monospace...`), so installed fonts still apply and offline machines fall back gracefully.
- The Tailwind CDN tag, the Font Awesome CSS link and the Google Fonts link are gone; the page makes **2 local requests, 0 external** in the runtime trap.

**Caveats recorded:** the icon glyphs are unicode substitutes rather than Font Awesome's artwork, and fonts fall back to system families when Inter/Fira Code are not installed. A future unit can vendor the woff2 files under SIL OFL if visual parity is required.

## 3. Stack-bound tokenizer artifact (`deepseek-52`, completed by `antigravity-60`)

The worker's earlier `UTF-8 byte-mod stand-in` mapping (`uor_model_worker.js`, `rawBytes[i] % vocabSize`) has been replaced by a stack-bound `ByteLevel` BPE tokenizer artifact (`schema: "uor-r4.studio-tokenizer/1"`, `mode: "byte-bpe"`):

- **`web/studio/assets/js/uor_tokenizer_artifact.js`** implements deterministic HuggingFace-compatible `ByteLevel` BPE encoding and decoding (`encodeTextWithArtifact`, `encodeBytesWithArtifact`, `decodeTokenIdsWithArtifact`, `decodeTokenIdWithArtifact`): GPT-2 `bytesToUnicode()` / `unicodeToBytes()`, leftmost-longest atomic `added_tokens` matching, optional `add_prefix_space`, rank-ordered `model.merges` merging, vocabulary lookup with `<|unk|>` fallback, and piece-to-UTF-8 decoding. It also bundles `DEFAULT_STACK_TOKENIZER_ARTIFACT` bound to the canonical `seed = 5` `mk-synthetic-stack` digest (`d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `vocab_size: 40`, matching `crates/uor-r4-integer/tests/m7_parity_library_cli_browser.rs`).
- **`web/studio/tools/make_tokenizer.mjs`** supports both `uor-r4.lut-stack/1` (`shape.vocab`) and `uor-r4.lut-discrete/1` (`shape.vocabulary`) headers, emits `mode: "byte-bpe"` into `pkg/tokenizer.json` by default, accepts `--source-tokenizer <hf-tokenizer.json>` to bind any trained BPE vocabulary/merge table to a stack's `stack_sha256`, and retains `--mode byte-mod` for legacy byte-mod comparisons.
- **`web/studio/assets/js/uor_model_worker.js`** resolves the tokenizer artifact on every `load_model` call (`payload.tokenizerArtifact`, `pkg/tokenizer.json`, or `DEFAULT_STACK_TOKENIZER_ARTIFACT`) without stale per-worker caching across reloads, labels bound BPE stacks as `"stack-bound byte-bpe tokenizer (vocab N, stack sha256 …)"` (dropping the `"stand-in"` label), encodes text prompts via `encodeTextWithArtifact`, decodes streamed pieces via `decodeTokenIdWithArtifact`, and falls back explicitly to `"UTF-8 byte-mod stand-in (rawBytes[i] % vocabSize; unbound - no matching artifact)"` when no artifact matches the loaded stack.
- **`web/studio/verify_studio_worker.mjs`** now auto-builds `pkg/uor_r4_integer_bg.wasm`, `pkg/uor_r4_integer.js`, and `pkg/synthetic-stack.lut` when missing on a clean checkout (`ensureWasmAndSyntheticStack()`) and verifies `byte-bpe` atomic role encoding, rank-ordered merges, `uor-r4.lut-discrete/1` header parsing, live worker text-prompt generation and piece decoding, and reload re-binding with zero network calls (`remoteRequests: 0`).

## 4. GitHub API calls

`web/studio/index.html:2866-2872` calls are user-initiated button flows, not page-load requests, and are already refused under Tauri/offline. Position recorded here: they fall outside the page-load bar; the static scan checks load references only. If the PM reads the bar as covering user-initiated calls, that is a product call (opt-in/removal), not changed silently.

## 5. Pages deploy

Stays paused (D36); this unit touches assets and verification only.
