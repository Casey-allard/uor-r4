# #718 — serving/#639-4: wire sentencepiece-unigram/2 through host consumers + complete differential fixtures

- **Type:** Issue
- **State:** closed (completed)
- **Author:** @Casey-allard
- **Created:** 2026-08-14T21:37:47Z
- **Updated:** 2026-08-15T00:00:37Z
- **Closed:** 2026-08-15T00:00:06Z
- **Assignees:** Casey-allard
- **URL:** https://github.com/UOR-Foundation/uor-r4/issues/718
- **Archived:** 2026-10-15T02:31:04Z · UOR-Foundation/uor-r4

## Body

References #639. This is the final planned increment of that parent; it does not auto-close #639. After this increment merges, post its delivery evidence on #639 and close the parent manually only if every acceptance item below is satisfied.

## Context

The first three #639 increments have landed:

- #705 / PR #706 pinned `google-t5/t5-base@a9723ea7…`, the model-card `apache-2.0` metadata, and the 791,656-byte `spiece.model`.
- #707 / PR #708 generalized the registry constructor to return `TokenizerModel`.
- #709 / PR #710 added the ModelProto parser and Unigram Viterbi core.
- #715 / PR #716 added the `nmt_nfkc` precompiled-charsmap normalizer and published `sentencepiece-unigram/1`.

Maintainer decision (2026-08-14): preserve published `sentencepiece-unigram/1` byte-for-byte and introduce reference-correct `sentencepiece-unigram/2` as the current definition. Auto-discovery selects `/2`; `/1` remains available only by explicit key for historical compatibility.

## Version and definition boundary

Both versions describe the raw 32,000-piece `spiece.model`, not the Hugging Face wrapper:

- `sentencepiece-unigram/1` is immutable historical behavior. It retains its published literal-`<unk>` sequence decode.
- `sentencepiece-unigram/2` retains the same raw model, normalization, segmentation, and token-id space, but decodes unknown pieces using `TrainerSpec.unk_surface`, matching the pinned reference implementation. It is the current/auto-selected version.
- Neither version implicitly inserts BOS/EOS or interprets a chat template.
- Source identity is `blake3(spiece.model)`.

At the pinned revision, Hugging Face `tokenizer.json` has 32,100 slots, 100 `<extra_id_*>` additions, and an EOS-appending post-processor. Those are distinct semantics and must fail closed unless a separately versioned adapter supports them.

Host-side encoding and deployed runtime behavior are also distinct:

- observation, evaluation, and teacher-backed serving use the exact registered host tokenizer;
- `tokenizer.bin` carries a bound decode table for the same id space;
- the deployed/WASM runtime does not run the f32/allocation-heavy Unigram Viterbi encoder;
- a decode-only tokenizer refuses prompt encoding instead of falling back to a legacy encoder.

## Scope

### 1. One registered source-tokenizer resolver

Add one fail-closed resolver used by every host consumer, with an explicit adapter key where a source exposes more than one definition.

- A valid byte-BPE `tokenizer.json` resolves to `hf-byte-bpe/1`.
- A valid raw `spiece.model` auto-resolves to current `sentencepiece-unigram/2`.
- Explicit `sentencepiece-unigram/1` remains resolvable for historical artifacts.
- A source containing both BPE and SentencePiece definitions is ambiguous without explicit selection.
- Absent, conflicting, malformed, present-invalid, unsupported, or unknown definitions return focused errors.
- Legacy `tokenizer.bin` remains reachable only through the explicit llama2.c checkpoint path.
- Preserve refusal by unrecognized family/version.

### 2. Thread the registered tokenizer through every consumer

Use the shared selection in:

- serial and batched `observe-text`;
- raw observe and source compile/export;
- evaluation token classification/decoding;
- teacher-backed serving prompt encode/decode;
- direct chat and public R4G1 bundle paths;
- observation and compiled-corpus adapter provenance/resume validation.

The worker/server trait-object surface must carry `Send + Sync` without changing legacy or byte-BPE behavior.

### 3. Truthful loss and byte-anchor semantics

SentencePiece normalization is not byte-length preserving.

- `encode_lossy` reports unrepresentable normalized input rather than always returning zero.
- SentencePiece observation records use unavailable source-byte anchors (`None`) unless an explicit original-input mapping exists.
- Runtime decode lengths are not relabeled as original source offsets.

### 4. Bound decode-only runtime tokenizer

Add an additive, tagged runtime representation for SentencePiece.

- It carries family, version, source tokenizer CID, adapter digest, and the full 32,000-id decode table.
- Whole-sequence decode matches the selected host adapter, including dummy-prefix and unknown-surface policy.
- `Tokenizer::encode_into` and `R4Engine::encode_text_into` refuse encoding for decode-only artifacts.
- Serving without the exact host encoder fails tokenizer-unavailable; it never borrows a legacy/global encoder.
- Existing untagged llama2.c and byte-BPE bytes parse exactly as before; byte-BPE export remains byte-identical.
- The final R4G1 HEAD tokenizer CID binds the exact exported `tokenizer.bin` bytes.
- Zero-CID compatibility is limited to historical absent/untagged tokenizers; tagged artifacts require an exact nonzero binding.
- Present-invalid sidecars, missing bound tokenizer bytes, mismatched graph/tokenizer pairs, and incompatible resumes fail before mutation or serving.

### 5. Comprehensive differential fixtures

Add a reproducible fixture generator pinned to the external `sentencepiece` package and a checked-in canonical table covering:

- ASCII and multilingual text;
- NFKC folds, fullwidth forms, ligatures, fractions, and Roman numerals;
- tabs/newlines and whitespace collapse;
- unknown single characters, adjacent unknown collapse, and separated unknown spans;
- control and unsupported piece types;
- empty input and malformed/out-of-range ids;
- encode ids and sequence decode;
- deterministic double-runs;
- both immutable `/1` and current `/2` identities/digests;
- unsupported ModelProto features and unknown registry versions;
- observation/evaluation/serving selection agreement;
- full id-space runtime decode parity;
- manifest persistence and incompatible-resume refusal.

The delivery record must include a non-vacuous run against the pinned source and identify exact Python package versions. A checked-in synthetic model keeps core selection, loss, export, and decode-only paths non-vacuous in ordinary CI.

## Non-goals

- No T5 teacher/executor or T5 model compilation.
- No Hugging Face T5 added-token/post-processor wrapper support under either raw SentencePiece version.
- No allocation-free/integerized SentencePiece encoder in the deployed/WASM runtime.
- No mutation of `sentencepiece-unigram/1`, `hf-byte-bpe/1`, legacy llama2.c tokenization, or historical artifact bytes.
- No implicit family detection from vocabulary size or special-token ids.

## Acceptance criteria

- [ ] Every host tokenizer consumer selects through one registered, fail-closed resolver.
- [ ] `sentencepiece-unigram/1` remains immutable and explicit-only; auto-selection resolves `sentencepiece-unigram/2`.
- [ ] New observations and compiled bundles record the exact `/2` adapter identity and refuse incompatible resume before mutation.
- [ ] SentencePiece unknown-input accounting is truthful and source-byte anchors are not fabricated.
- [ ] Tagged runtime artifacts decode the bound id space and refuse encoding by construction.
- [ ] Missing host SentencePiece models cannot fall back to legacy/global prompt encoding.
- [ ] R4G1, tokenizer bytes, runtime embedded identity, source adapter, and provenance sidecars agree or fail closed.
- [ ] The pinned reference differential covers encode, sequence decode, policy, determinism, and the complete runtime decode table.
- [ ] The real T5 `spiece.model` run is non-vacuous and reproduces its recorded source identity.
- [ ] HF wrapper semantics are rejected rather than approximated.
- [ ] Existing SmolLM2/GPT-2 byte-BPE and llama2.c tokenizer bytes/tests are unchanged.
- [ ] Documentation states the host-encode/runtime-decode and raw-SentencePiece/HF-wrapper boundaries.

## Verification

```bash
cargo test -p uor-r4-core --offline \
  --test t5_tokenizer_pin \
  --test sentencepiece_unigram \
  --test tokenizer_adapter \
  --test sentencepiece_reference_fixture -- --nocapture
python3 scripts/gen_t5_sentencepiece_fixture.py --check

cargo test -p uor-r4-graph-compiler --offline
cargo test -p uor-r4-graph-cli --lib --offline
cargo test -p uor-r4-api --offline
cargo test -p uor-r4-wasm-router --lib --offline
cargo check --target wasm32-unknown-unknown -p uor-r4-wasm-router --lib
python3 scripts/check_claim_wording.py
cargo fmt --check
cargo clippy --workspace --all-targets --all-features --offline -- -D warnings
cargo test --workspace --offline
cargo check -p uor-r4-graph-format --no-default-features
cargo check -p uor-r4-graph-format --no-default-features --features alloc
```

## Delivery evidence required on #639

Post:

- merged PR and merge commit;
- exact source revision, byte count, SHA-256, source CID, and `/1` + `/2` adapter digests;
- exact reference package versions;
- differential case/count summary and proof that no presence-gated test printed `UNAVAILABLE`;
- byte-BPE/legacy byte-regression results;
- runtime encode-refusal and full decode-parity results;
- graph/tokenizer/sidecar binding and incompatible-resume evidence.

Only then close #639 manually.

## Comments (3)

### comment 5298492456 — 2026-08-14T21:47:15Z — Casey-allard

## Non-vacuous decode differential: adapter-version decision

The pinned T5 source is present locally and the pin gate passes non-vacuously (791,656 bytes; SHA-256 `d60acb128cf7b7f2536e8f38a5b18a05535c9e14c7a355904270e15b0945ea86`; 2/2 tests).

Reference `sentencepiece==0.2.2` exposed one behavior not covered by #716's encode-only fixtures: T5 id 2 (`<unk>`) decodes as ` ⁇ ` (U+2047 with surrounding spaces), while the currently registered `/1` implementation concatenates the literal surface `<unk>`. Out-of-range decode policy also needs an explicit contract.

Two maintainer choices:

1. **Add `sentencepiece-unigram/2` (recommended).** Freeze `/1` exactly as published, add reference-correct decode semantics under `/2`, retain both registry entries, and make `/2` the current consumer selection. This moves the new adapter/delivery identities but preserves immutable registry history.
2. **Correct `/1` in place.** Simpler, and no production observe/serve consumer currently emits `/1`, but it changes an already published registry entry and weakens the immutability rule used elsewhere.

Version-neutral runtime-container and provenance work can proceed while this is decided; the final adapter identity and differential pins will wait for the maintainer choice.


### comment 5299096360 — 2026-08-14T23:17:55Z — Casey-allard

Maintainer decision resolved: **Choice A**.

Implementation contract now in the working branch:

- freeze published `sentencepiece-unigram/1` as explicit-only historical behavior;
- introduce reference-correct `sentencepiece-unigram/2` as current;
- auto-discovery selects `/2`;
- keep the raw 32,000-piece model distinct from the 32,100-slot Hugging Face wrapper.

Pinned non-vacuous identities for `google-t5/t5-base@a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1`:

- bytes: `791,656`
- SHA-256: `d60acb128cf7b7f2536e8f38a5b18a05535c9e14c7a355904270e15b0945ea86`
- source CID: `blake3:18af7072534a34d626ca166c92f5e37c7ccfcd6ce6877c3c25d325bc8201090d`
- immutable `/1` adapter digest: `blake3:cc9b3d1ab746c3ba0cd1f4b94a6b9306a3bd8a41fe9e539800c538a2cee7d8ab`
- current `/2` adapter digest: `blake3:e56dc74051fdaf4a90a626328ad01c81b7cd26044b2b071b8d50302c07bbacd1`

The external fixture is pinned to `sentencepiece==0.2.2`, `blake3==1.0.9`, and `protobuf==6.33.6`. The focused real-source acceptance suite is currently non-vacuous and green: T5 pin 2/2, SentencePiece 4/4, adapter registry 18/18, and external reference fixture 3/3, including the complete 32,000-id decode table. Full workspace/target gates and the independent final diff review are still in progress; final delivery evidence will be posted after the merge-queue verdict.

### comment 5299332003 — 2026-08-15T00:00:37Z — Casey-allard

## Delivery evidence — merged

PR #719 merged through the protected merge queue at a3def9dbb63f3ea54f248a02a61606ee6fd44905.

- PR: https://github.com/UOR-Foundation/uor-r4/pull/719
- Binding speculative run: https://github.com/UOR-Foundation/uor-r4/actions/runs/31851321265
- Maintainer choice A is implemented: sentencepiece-unigram/1 is immutable explicit-only history; reference-correct sentencepiece-unigram/2 is current and auto-selected.
- Pinned raw T5 source: revision a9723ea7f1b39c1eae772870f3b547bf6ef7e6c1, 791,656-byte spiece.model, source CID blake3:18af7072534a34d626ca166c92f5e37c7ccfcd6ce6877c3c25d325bc8201090d.
- Current /2 adapter digest: blake3:e56dc74051fdaf4a90a626328ad01c81b7cd26044b2b071b8d50302c07bbacd1.

Delivered across every in-scope consumer: explicit CLI/API/server selection, source resolution, raw/text observation, compile and evaluation, atomic adapter provenance, tagged decode-only runtime export, graph HEAD tokenizer-CID binding, serving/chat/public bundle fail-closed behavior, and deterministic external fixtures. The raw 32,000-piece SentencePiece model remains explicitly distinct from the 32,100-slot Hugging Face wrapper.

Verification was non-vacuous with the pinned source present: T5 pin 2/2, SentencePiece adapter 4/4, tokenizer registry 18/18, external reference fixture 3/3 including the complete singleton table, graph compiler 104/104, graph CLI 44/44, API 29 passed with one unrelated ignored test, and root library 72/72. Full workspace, all-features, no_std, wasm, κ, Gate C, fuzz, cargo-audit, and R1-R6 conformance gates all passed in the merge queue. The reference fixture reported no UNAVAILABLE path.


---
_Comment/review ids in this file: 5298492456, 5299096360, 5299332003_

