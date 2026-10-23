# REPORT: m7-studio-self-host-page-assets, deepseek-52 (stack-bound tokenizer artifact)

Lab: deepseek. Order: [ORDER.md](ORDER.md). Item #3075 (residual of #2868). Base: 74b5a9538. References #3075.

## What did not work, first

- **The worker called its mapping a tokenizer while it was unbound.** encodePromptTokens mapped prompt bytes as rawBytes[i] % vocabSize and the capability label said UTF-8 byte-mod stand-in; the item asks for a **stack-bound** artifact. The mapping is still byte-mod (a real corpus tokenizer needs a real model), but it is now generated from the shipped UORLUT01 header, tied to the stack's sha256 and vocabulary, and labelled with both.
- **The artifact cannot be committed with the bundle.** web/studio/pkg is wasm-pack build output (gitignored), so the artifact is generated at build time from the same stack file the page ships; the tool supports --check for a post-build or CI assertion, and the worker falls back explicitly when it is absent.
- **JSON module import needed care in the verification harness.** The worker loads the artifact with a same-origin dynamic JSON import and attributes; the Node worker harness runs the real module, so the verification script writes the artifact beside the bundle before importing the worker, then asserts the live capability label rather than trusting the helper alone.

## What changed

| File | Change |
|---|---|
| web/studio/tools/make_tokenizer.mjs | new build tool: parses the UORLUT01 header (magic, LE u64 length, JSON schema/shape.vocab), emits pkg/tokenizer.json (schema, mode, stack sha256, vocabulary, 256-entry table), --out/--check modes |
| web/studio/assets/js/uor_tokenizer_artifact.js | new helper: schema constant, sha256Hex, validateTokenizerArtifact, tokenizerLabel, encodeBytesWithArtifact (shared by worker, tool and verification) |
| web/studio/assets/js/uor_model_worker.js | loads the artifact on load_model, validates it against the loaded stack, uses its table for prompt bytes, reports the stack-bound label with an explicit unbound fallback |
| web/studio/verify_studio_worker.mjs | generates the artifact from the stack it reads, writes it beside the bundle, asserts the stack-bound label, and unit-asserts the fallback label + encoding |
| docs/products/web-client.md | tokenizer capability description now stack-bound with the explicit fallback |
| docs/labs/m7-studio-self-host-page-assets/README.md | section 3 records the implemented artifact, the stand-in honesty note, and the real-tokenizer dependency on M4 |
| this folder | ORDER and REPORT |

## Measurements

- Bundle: wasm-pack build with RUSTFLAGS=-C target-feature=-simd128 (release wasm 381,818 B), synthetic stack seed 5 (469,252 B).
- Artifact: pkg/tokenizer.json -> vocab_size 40, stack_sha256 d60f88d6e08a… (full digest d60f88d6e08ac60763b35484307f9425f702b9fead4615dd8cbe5fdbbced92df), mode byte-mod, 256-entry table; --check against the stack: TOKENIZER ARTIFACT MATCHES THE STACK.
- Live worker verification (node web/studio/verify_studio_worker.mjs): ALL STUDIO WORKER CHECKS PASSED; tokenizerLabel = "byte-mod stand-in, bound to stack sha256 d60f88d6e08a… (vocab 40)"; the 32-token trajectory is byte-identical to the previous unbound mapping (same table formula, now stack-bound); save/restore bit-identical; trap self-test 6/6; remoteRequests 0.

## Checks run at the head

node web/studio/verify_studio_worker.mjs on a freshly built bundle and synthetic stack; node web/studio/tools/make_tokenizer.mjs <stack> --check <artifact>; uor-harness check, loop report --check, loop pr-check --local, loop gates --full; counted non-author record (code PR).

## Skills used

- solution-integrity: made the artifact validate against the actual shipped stack (sha/vocab/table) instead of accepting any JSON; kept the fallback path and its label honest; tested the live worker label, not just the helper.
- scope-guard: web/** + docs only; no Rust, no Python, no Pages, no bar.

## Still open

- **This binds the stand-in; it does not replace it:** the item done-when stays open until a real corpus tokenizer exists on a real model (M4 export). The runtime page-load trap and the demo-page decision also remain on this item (#3075).
- Known limits recorded from review: real D11 exports are uor-r4.lut-discrete/1 with shape.vocabulary (deliberately refused by this tool until an M4-aware mode lands); the worker's dynamic JSON import is cached per worker, so a second stack in one worker revalidates against the first artifact and falls back to the unbound label (safe).

## Plain summary

- The Studio's prompt tokenizer is now a stack-bound artifact generated from the shipped stack header, loaded and validated by the worker, with the stack digest and vocabulary in its label and an explicitly-unbound fallback. It is still a byte-mod stand-in for the synthetic model, which the record says plainly.
