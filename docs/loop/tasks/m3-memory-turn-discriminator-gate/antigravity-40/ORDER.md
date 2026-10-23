# ORDER: m3-memory-turn-discriminator-gate, antigravity-40

Lab: antigravity. Item: [m3-memory-turn-discriminator-gate](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md). Base: `2365778e9`.

## Goal

Repair PR #3079 (`antigravity-39`) against the exact-head `CHANGES` review (`6368666608` at `2d6c597e5539daed4d57282e57e401e759258217`) for item `m3-memory-turn-discriminator-gate` (#2855):

> The last clause of the M3 bar in `docs/integration/project-track.md:116-120`: non-memory conversational prose does not silently overwrite stored facts. It is measured on the pre-registered panel.

## Cancel probe (run first)

Command:

```sh
~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler::tests::saved_compiler_action_wires_configurable_gate_and_real_margin
```

Result on base `2d6c597e5` before the `antigravity-40` repair: `running 0 tests` (`SavedCompiler` was behind `#[cfg(any())]` with a hardcoded `MemoryTurnGate::default()` and `None` margin in `SavedCompiler::action` / `SavedCompiler::table_action`, and the `world=v2` template test checked only 29 hand-typed strings rather than all 79 `milestone_world_v2::relations()` assert+update templates across `Split::Train` and `Split::Development`).

## Files in scope

- `crates/uor-r4-training/**`
- `docs/compute/**`
- `docs/labs/m3-memory-turn-discriminator-gate/**`

## Done when

- `SavedCompiler` is active in `crates/uor-r4-training/src/relation_compiler.rs`, carries a configurable `gate: MemoryTurnGate` field (defaulting to `MemoryTurnGate::disabled()` on `load`/`fit` so the un-gated baseline arm is preserved and configurable via `with_memory_turn_gate` / `set_memory_turn_gate`), and computes a real `StatementMargin` from `relation_head`, `act_head`, and `span_head` linear logits on `source` inside `SavedCompiler::table_action_with_margin` and `SavedCompiler::action`.
- `memory_turn_gate_preserves_all_world_v2_relation_statement_templates` iterates directly over `milestone_world_v2::relations()` across `Split::Train` and `Split::Development` (`79 / 79` assert+update templates plus paraphrase variants).
- `has_memory_turn_cue` and `has_third_person_possessor` use general syntactic rules (no panel-specific noun blocklist) and reject negated statements, conditional/hypothetical turns, third-person reported speech, uncapitalized `I'm <adjective>` small talk against `user_name`, and narrative/past-tense `code_word` frames while preserving all `79 / 79` `world=v2` statement templates.
- `saved_compiler_action_wires_configurable_gate_and_real_margin` pins `SavedCompiler::action` end-to-end so reverting the wiring fails the test, and `REPORT.md` / `README.md` accurately report diff stats and distinguish host synthetic-adversary checks from the pre-registered GPU measurement.

## Stop and ask if

- Any file outside the `ORDER.md` scope must change.
- Any GPU evaluation or bar/spend registration is needed (awaits non-author `TEST FITNESS: FIT` and owner release on `#2855`).
