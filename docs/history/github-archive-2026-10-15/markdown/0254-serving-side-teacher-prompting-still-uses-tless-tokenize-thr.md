# #254 — Serving-side teacher prompting still uses tless_tokenize — thread the HF BPE tokenizer through the serving path (#242 follow-up)

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-07-28T05:04:08Z
- **Updated:** 2026-07-29T18:06:47Z
- **Closed:** 2026-07-29T18:06:47Z
- **Labels:** bug
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/254
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

### Summary
Follow-up to #242 (merged in #253). Observation/evaluation now use the model's real byte-level BPE (`HfBpeTokenizer`), but **serving-side teacher prompting still tokenizes with the bespoke `tless_tokenize`**: `generate_attention_text` formats `"User: {prompt}\nAssistant:"` and encodes it with the legacy tokenizer before feeding the teacher oracle. This is the same mismatch class that caused the Jul 26 ChatML fragmentation bugs (worked around by dropping ChatML) and the #242 distortion.

TODO markers at the exact spots: `src/server.rs:712` (`generate_attention_text`) and `src/tless_uor.rs:542`.

### Why not done in #253
The serving path runs through the no-alloc `tless_tokenize / tless_tokenize_into / tless_detokenize(_into)` surface with ~10 call sites in `server.rs`; threading `TokenizerKind` through it is a contained but real refactor, too large to ride along a compiler-path PR.

### Scope
- Teacher-oracle prompting (and its decode) uses the model's `HfBpeTokenizer` when serving an HF model.
- Re-evaluate restoring the native ChatML instruct template for SmolLM2 (dropped in `9f9e525` purely because of the fragmentation this fixes) — plain-text `User:/Assistant:` framing costs instruct-tuned models quality.
- The r4g1/transformerless engines' own tokenization is a separate question (their artifacts are compiled with the observation tokenizer — after #253, HF artifacts and serving must agree; verify `tokenizer_cid` checks catch any mismatch).

### DoD
Teacher responses served with the model's own tokenizer end-to-end; ChatML template restored (or a measured decision not to); no regression in the serving cascade; tokenizer_cid verification passes.


## Comments (1)

### comment 5120731140 — 2026-07-29T16:35:58Z — Casey-allard

## Implementation plan (assigned; branch `issue-254-serving-bpe`)

**Defect surface** (measured lineage: #242/#253): `generate_attention_text` (src/server.rs:705-771) encodes the prompt with `tless_tokenize` and decodes output with `tless_detokenize_into` — the legacy llama2.c id space — while the HF teacher (SmolLM2) lives in the HF BPE id space. Same class of mismatch #253 fixed on the observation path (agreement 2.6%→23.1% there).

**Change**: load an `HfBpeTokenizer` (uor-r4-core `transformerless::hf_bpe`, #253) from the oracle source dir's tokenizer.json at oracle-load time; thread it into `generate_attention_text`; encode seed + decode generation through it when present, with the legacy tless pair as explicit fallback for local-llama oracles (`tokenizer_address()` distinguishes). The `address()` of the tokenizer goes into the serving telemetry for attestation consistency (#260 lineage). No runtime-kernel change; serving-side only.

**DoD**: attention-tier responses from a SmolLM2 oracle round-trip through the HF id space (spot-check: prompt tokens re-decode to the prompt); gates + queue green.


---
_Comment/review ids in this file: 5120731140_

