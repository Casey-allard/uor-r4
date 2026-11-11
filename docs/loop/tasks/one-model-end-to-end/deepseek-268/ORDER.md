# ORDER: one-model-end-to-end, deepseek-268 (scenario corpus generator)

Lab: deepseek. Item: [one-model-end-to-end](../../../../roadmap/items/one-model-end-to-end.md) (#3407). Base: `c0f7db014`.

## Goal

Build a deterministic Rust generator and checker for aligned dialogue scenarios. One
scenario seed must produce the statement, question wording, distractor facts, expected
value, reference reply, response mask and split metadata together. The output must use
the existing `uor-r4-chat-corpus/v1` train format so no trainer or artifact change is
required.

This unit does not train a model or alter an active model, trainer, exporter or serving
path.

## Cancel probe

Read the existing M-world corpus and manifest. At the base, the corpus has substantial
Wikipedia rows without questions, and no manifest ties each row to a paraphrase family,
answer value and expected EOS independently. The cancel line is met if the existing
generator already emits all those bindings and no row without a scoreable question.
If it does, stop without writing a second generator.

## Files in scope

- `crates/uor-r4-training/src/scenario_corpus.rs`
- `crates/uor-r4-training/src/bin/scenario-corpus.rs`
- `crates/uor-r4-training/tests/scenario_corpus.rs`
- `crates/uor-r4-training/src/lib.rs`
- `crates/uor-r4-training/Cargo.toml`
- `docs/roadmap/items/one-model-end-to-end.md`
- `docs/research/notebook/data-tokenizer.md`
- `docs/loop/tasks/one-model-end-to-end/deepseek-268/**`

Size budget: 900 changed lines. If the work exceeds 1,200, stop and report.

## Done when

1. The generator supports the memorise, paraphrase, selection, update, abstention,
   copy and general-dialogue families, plus no-memory, wrong-memory and stale-memory
   controls.
2. Train, development and fresh scenarios are disjoint by scenario id, entity name,
   value tuple and question-template family where those holdouts are available.
3. Every generated reply is response-masked through EOS, and every scored question has
   one authoritative answer value.
4. The generator writes `tokens.u16`, `response_mask.u8`, a trainer-compatible
   `manifest.json`, `scenarios.jsonl` and a top-level `scenario-manifest.json`.
5. `scenario-corpus check` regenerates and refuses byte, hash, label, mask or split
   mismatches.
6. Tests pass for deterministic bytes, disjoint splits, tokenizer round-trip,
   representative answer values and all negative mutations.
7. A small local smoke corpus is generated; large corpus files and binaries are not
   committed.

## Stop and ask if

- A new dependency is needed.
- The existing corpus ABI must change.
- A file outside scope must change.
- The generator cannot preserve label/mask identity through the pinned tokenizer.

STOP and report when done. Do not start the next task.
