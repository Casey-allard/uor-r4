# ORDER: one-model-end-to-end, deepseek-39 (CPU-readable SFT mix manifest)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `fc2d2b143`.

## Goal

Make `mix-chat-corpus` emit a complete `uor-r4-chat-corpus/v1` manifest that
`uor-r4-cpu-train` can validate directly, so the laptop trainer and CPU
diagnostics can consume the exact reply-SFT mix without hand-editing metadata.

## Cancel probe

- `cpu-point-transfer-driver` refuses the sealed SFT mix with
  `corpus manifest vocabulary differs`.
- The mix manifest omits `tokenizer.vocab_size`, `tokenizer.bos_id` and
  `max_tokens`; the CPU validator requires all three.
- The source tokens and mask validate, and the missing producer metadata needs
  no model compute.

## Files in scope

- `crates/uor-r4-training/src/bin/mix-chat-corpus.rs`
- `crates/uor-r4-training/tests/mix_chat_corpus_manifest.rs`
- `crates/uor-r4-training/Cargo.toml`
- `Cargo.lock`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-39/**`
- `docs/research/notebook/cpu-training.md`

## Done when

1. `mix-chat-corpus` requires compatible tokenizer identity/vocabulary metadata
   from every input and records the complete output tokenizer metadata.
2. The output records a positive `max_tokens` covering every input document and
   a valid BOS value.
3. A regression test invokes the real binary on synthetic UORT sources, proves
   token/mask preservation, and passes the resulting manifest through
   `uor_r4_cpu_train::validate_corpus_manifest`.
4. Mismatched input tokenizer hashes or vocabulary metadata are refused before
   an output is claimed.
5. The relevant training and CPU tests, report check, policy check and full
   gates pass; one reviewed PR merges.

## Stop and ask

Do not alter token bytes, masks, source ordering, count semantics or the sealed
SFT input. Do not train or use a GPU in this unit.
