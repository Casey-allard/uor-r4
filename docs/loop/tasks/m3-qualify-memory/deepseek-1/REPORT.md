# REPORT: m3-qualify-memory, deepseek-1

Lab: deepseek. Item:
[m3-qualify-memory](../../../../roadmap/items/m3-qualify-memory.md).
Order: [ORDER.md](ORDER.md). References #2031.

## What did not work, first

- The preceding explicit-memory probe showed that both public artifacts consume
  `System: Memory:` but do not answer with the supplied `Pickle` or tomatoes.
  This unit therefore tests extraction and protocol integration only; it cannot
  turn that model-quality failure into a capability.
- The initial `call` extraction path treated "call me Captain Sam" as an
  ordinary value lookup and retained `me`. It now recognizes `call` as a name
  request and skips the indirect object.
- Automatic memory is intentionally not default-on. Enabling it by default
  would change the merged chat baseline before the new behavior has a product
  result.
- The six-process automatic-memory smoke did not produce model recall. Both
  seeds emitted the correct `Pickle` and tomatoes memory lines, but neither
  model answer contained its supplied value.

## What changed

Added `stack/dialogue_memory.rs`, a deterministic serving-side identity-channel
sieve. It reads prior user turns only, splits them into clauses, ranks clauses
by shared normalized content atoms and recency, then extracts bounded name,
temporal, locative, age or default predicate values.

The stack API exports `recall` and `recall_value`; `uor-chat` gains the opt-in
`--memory-sieve` flag and `/memory auto` / `/memory manual`. Explicit
`/memory <VALUE>` overrides the sieve for one turn. The fact log is cleared by
`/reset`, and generated assistant messages are never logged as facts.

The code change is confined to `crates/uor-r4-integer` plus task documentation.
It adds no training, tokenizer, artifact-format or numeric-kernel change.

## Measurements

Pure extraction controls cover the two fixed public probes, a correction,
named/called forms, where, when, age, color and rename questions, no-match
questions and pronoun-only queries.

The FIT/PRE-RUN-PASS host smoke then ran six fresh processes: two fixed
multi-turn conversations on each of the two public seeds, plus an exact seed-1
repeat. All 6 exited 0 with empty stderr. All 15 replies used protocol 2 and
had valid stops (10 EOS, 5 token-cap). Both seeds emitted `Memory: Pickle.`
before the first answer and `Memory: tomatoes.` before the second. The seed-1
repeat matched the first pass after excluding only the registered `id/s`
timing field.

No factual reply contained its supplied value: **0/6**. This is
`AUTO_MEMORY_OBSERVED` with `MODEL_RECALL_NOT_OBSERVED`. Peak RSS was 11,888
KiB and process wall time was 0.17–0.22 seconds. The public artifacts remain
deterministic smoke targets, not useful chat. Full record:
[lab result](../../../../labs/m3-qualify-memory/deepseek-1/README.md).

## Checks run at the head

- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib
  dialogue_memory`: 5 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --bin
  uor-chat`: 5 passed.
- `cargo test --locked -p uor-r4-integer --features serving-wasm --lib`:
  442 passed, 1 ignored, 0 failed.
- `uor-harness prep-check`: FIT, complete, `may_compute=true`.
- Targeted `rustfmt --check` on the new sieve and changed CLI: passed.
- `uor-harness loop report --check`: passed.
- `uor-harness check`: 0 errors and 52 unrelated project-wide warnings.
- Independent fresh-context review: `TEST FITNESS: FIT`, `PRE-RUN: PASS`.
- `git diff --check`: passed.

## Skills used

None of the available Codex skills applied. The current charter, M3 item,
training sieve, serving bridge and product code were read directly.

## Still open

- Obtain an independent FIT/PRE-RUN review, then run the six fixed automatic
  memory conversations and record their exact evidence. **Done.**
- Automatic extraction coverage beyond the bounded lexical forms remains
  unmeasured.
- The in-memory fact log is unbounded while automatic mode is off; recognized
  questions without a match request `Memory: none.`; and the fact log survives
  the CLI's internal context-overflow reset. These are nonblocking for this
  fixed smoke but need hardening before general automatic mode.
- Model answer quality and M3 qualification remain open.
- ARM64/WebAssembly audit qualification and GPU export/parity work remain
  outside this host-only support unit.

## Plain summary

The chat program can now search what the user said earlier, select the most
relevant clause, extract a short value, and feed that value into the memory
format the model expects. Both seeds received the right value automatically,
but neither model used it in an answer. This works only for bounded patterns;
it is not a general reader and does not close M3.
