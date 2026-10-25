# ORDER: m7-studio-self-host-page-assets, antigravity-60

Lab: antigravity. Item: [m7-studio-self-host-page-assets](../../../../roadmap/items/m7-studio-self-host-page-assets.md). Base: `e6f261600`.

## Goal

The item's done-when reads:

> `web/studio/index.html` and (if kept) the router demo page make zero external requests on load, checked by the extended trap verification; the stack-bound tokenizer artifact replaces the stand-in; the GitHub API calls' position is recorded in the item's record; `docs/products/web-client.md` updated to match.

## Cancel probe (run first)

Ran the page-load and worker verification suite on the clean checkout at `e6f261600e8f6f2cf90afc5659cef9dc8dedeffb`:
1. `node web/studio/verify_studio_page.mjs --strict-router` -> `PAGE-LOAD CHECKS PASSED` (`clean: ["geometric_prime_router_webapp.html", "index.html"]`).
2. `node web/studio/verify_studio_page_load.mjs --strict-router` -> `RUNTIME PAGE-LOAD TRAP PASSED` (`index.html` & `geometric_prime_router_webapp.html`: 0 external requests).
3. `node web/studio/verify_studio_worker.mjs` -> **FAILED** (`ENOENT` on `web/studio/pkg/uor_r4_integer_bg.wasm`), and `uor_tokenizer_artifact.js` / `make_tokenizer.mjs` only implemented `mode: "byte-mod"` (`rawBytes[i] % vocabSize`, retaining the `"byte-mod stand-in"` label).

- **Cancel-probe result:** **DID NOT CANCEL** (`instrument`). Page-load traps pass; `verify_studio_worker.mjs` needs clean-checkout WASM/stack bootstrap and `uor_tokenizer_artifact.js` / `make_tokenizer.mjs` / `uor_model_worker.js` need the bundled stack-bound `ByteLevel` BPE tokenizer (`mode: "byte-bpe"`).

## Plan (expected files > 3)

1. `web/studio/assets/js/uor_tokenizer_artifact.js`: implement `mode: "byte-bpe"` (`ByteLevel` BPE vocabulary, rank-ordered merges, atomic `added_tokens`, piece encoding/decoding) alongside `mode: "byte-mod"`, bundle `DEFAULT_STACK_TOKENIZER_ARTIFACT` bound to the canonical seed-5 `mk-synthetic-stack` digest (`d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df`, `vocab_size: 40`), and replace the `"byte-mod stand-in"` label when bound.
2. `web/studio/tools/make_tokenizer.mjs`: support `uor-r4.lut-stack/1` and `uor-r4.lut-discrete/1` headers, default to `mode: "byte-bpe"` (with `--source-tokenizer` and `--mode byte-mod`), and validate both modes under `--check`.
3. `web/studio/assets/js/uor_model_worker.js`: resolve and validate the tokenizer per loaded stack without stale per-worker caching across reloads, encode text prompts through `encodeTextWithArtifact`, and decode generated tokens through `decodeTokenIdWithArtifact`.
4. `web/studio/verify_studio_worker.mjs`: add `ensureWasmAndSyntheticStack()` for clean checkouts and verify bound `byte-bpe` encoding, piece decoding, rank-ordered merges, `uor-r4.lut-discrete/1` headers, and reload re-binding.
5. `docs/products/web-client.md`, `docs/labs/m7-studio-self-host-page-assets/README.md`, and `docs/roadmap/items/m7-studio-self-host-page-assets.md`: update documentation and set `status: done`.

## Files in scope

- `web/**`
- `docs/products/**`
- `docs/labs/m7-studio-self-host-page-assets/**`

## Done when

`node web/studio/verify_studio_page.mjs --strict-router`, `node web/studio/verify_studio_page_load.mjs --strict-router`, and `node web/studio/verify_studio_worker.mjs` pass on a clean checkout with `remoteRequests === 0`; the stack-bound `ByteLevel` BPE tokenizer replaces the `byte-mod` stand-in on bound stacks; the GitHub API calls' position is recorded in `docs/labs/m7-studio-self-host-page-assets/README.md` and `docs/products/web-client.md`; and `uor-harness loop gates --full` passes.

## Stop and ask if

- Never wait on Casey or another lab (`CHARTER.md` §5.1, D65–D69). Stop only if a change requires editing an owner-only path.
