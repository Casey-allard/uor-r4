# ORDER: m3-qualify-memory, deepseek-1

Lab: deepseek. Item: [m3-qualify-memory](../../../../roadmap/items/m3-qualify-memory.md). Base: `8cf219d4575203e8ff664d65de11ecaecbb50caf`.

## Goal

Add an opt-in deterministic serving-side memory sieve. It reads prior user
turns, selects an exact lexical clause that shares the query's content atoms,
extracts a bounded value, and feeds the already-tested `System: Memory:` bridge.

This is host-side serving support, not M3 qualification and not evidence that
the model answers correctly. The selected public artifacts remain deterministic
smoke targets only.

## Scope

- `crates/uor-r4-integer/src/stack/**`
- `crates/uor-r4-integer/src/bin/uor-chat.rs`
- `crates/uor-r4-integer/README.md`
- `docs/harness/prep/m3-qualify-memory-deepseek-*`
- `docs/labs/m3-qualify-memory/deepseek-1/**`
- `docs/loop/tasks/m3-qualify-memory/deepseek-1/**`
- `docs/research/notebook/m3-durable-memory.md`
- `docs/roadmap/items/m3-qualify-memory.md`

## Required design

- Keep automatic extraction default-off; preserve existing chat behavior.
- Expose `--memory-sieve` and REPL `/memory auto` / `/memory manual`.
- Use exact deterministic text rules: prior user turns only, clause selection
  by shared normalized content atoms plus recency, and bounded keyword-to-value
  extraction for name/call, temporal, locative, age and direct predicate forms.
- Never use model attention scores, floating point, softmax, squashing or matrix
  multiplication in the sieve. Never treat generated assistant text as a fact.
- Explicit `/memory <VALUE>` overrides automatic extraction for one turn.
- Clear the transcript memory on `/reset`.
- Prove exact positive and negative extraction cases, including:
  - the puppy-name and tomato probes;
  - a later correction replacing an earlier value;
  - unrelated and pronoun-only queries producing no value;
  - generated assistant text never entering the fact log;
  - explicit memory overriding the sieve;
  - reset clearing both facts and queued explicit memory.

## Done when

- The sieve and CLI controls are implemented and unit-tested.
- The two fixed public memory probes are extracted correctly before model use.
- No-match and correction controls pass.
- No trained-model run happens until a separate exact-command pre-registration
  receives FIT/PRE-RUN.
- A REPORT, notebook row and reviewed PR record what ran and what remains.
