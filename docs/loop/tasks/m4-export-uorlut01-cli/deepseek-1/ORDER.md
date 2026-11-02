# ORDER: m4-export-uorlut01-cli, deepseek-1

Lab: deepseek. Item: [m4-export-uorlut01-cli](../../../../roadmap/items/m4-export-uorlut01-cli.md). Base: `e0647330314841ddf84ba3bd7cfd57e57c03680e`.

## Goal

Add the missing system-memory turn to the integer stack-chat path. Training
places `System: Memory: {value}.` between a recall question and the assistant
marker; product `uor-chat` currently cannot encode that turn. This unit adds a
serving bridge for an explicit value and proves its token layout against the
same protocol encoder.

This unit does not invent the value. Automatic statement extraction remains the
next layer. It exposes the exact contract that an extractor must feed and
allows a separately registered product run to test whether the trained model
uses the line.

## Scope

- `crates/uor-r4-integer/src/stack/chat.rs`
- `crates/uor-r4-integer/src/stack/tests.rs`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `crates/uor-r4-integer/README.md`
- `docs/harness/prep/m4-export-uorlut01-cli-memory-bridge-*`
- `docs/harness/claims.json`
- `docs/labs/m4-export-uorlut01-cli/deepseek-1/**`
- `docs/loop/tasks/m4-export-uorlut01-cli/deepseek-1/**`
- `docs/research/notebook/m4-serving.md`
- `docs/roadmap/items/m4-export-uorlut01-cli.md`

No training, compiler, store, tokenizer, artifact-format or model change.

## Required design

- Add one `StackChat` reply entrypoint that accepts an optional plain memory
  value and encodes the turn as user, then
  `System: Memory: {value}.`, then the assistant marker.
- Keep the existing no-memory `reply` behavior byte-identical by routing it
  through the new entrypoint with no value.
- Add one-shot `--memory <VALUE>` and REPL `/memory <VALUE>` controls. A queued
  value applies to exactly the next user turn; `/memory` reports it and
  `/memory clear` clears it.
- Reject empty values, CR/LF and literal special tokens before model mutation.
- Record the exact system line in one-shot JSON. Do not add memory to
  `--prompt-ids`, retained-bundle, non-stack or `--inspect` paths.
- Test the exact encoded token sequence under protocol 2, no-memory equivalence,
  one-turn consumption, clearing and invalid input. No dense matmul, softmax,
  squashing, float serving or new numeric kernel is allowed.

## Done when

1. The stack-chat API and CLI controls above are implemented.
2. Focused tests pass for protocol-2 layout, no-memory behavior, rejection and
   one-turn queue semantics.
3. The integer crate builds with `--features serving-wasm`; the serving audit
   remains clean for the changed path.
4. A separate pre-registration is FIT-reviewed before any trained-model prompt
   uses the new path.
5. A REPORT, one notebook row and a reviewed PR record what ran and what
   remains automatic-extractor work.
