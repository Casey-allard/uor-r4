# ORDER: m3-memory-turn-discriminator-gate, antigravity-39

Lab: antigravity. Item: [m3-memory-turn-discriminator-gate](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md). Base: `b8aa3aefd`.

## Goal

The item's done-when reads:

> The last clause of the M3 bar in `docs/integration/project-track.md:116-120`: non-memory conversational prose does not silently overwrite stored facts. It is measured on the pre-registered panel.

## Cancel probe (run first)

Command:

```sh
~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler::tests::memory_turn_discriminator_and_margin_gate_controls
```

Result on base `b8aa3aefd` before adding the memory-turn discriminator and margin gate: `running 0 tests` (`SavedCompiler::action`, `SavedCompiler::table_action`, and `resolve_op_policy` contain no margin or memory-turn discriminator gate, so under `OpPolicy::UnlessQuery` a small-talk / anaphoric / near-miss turn where the op model emits `Op: assert ...` or `Op: update ...` compiles directly to `CompiledAction::Assert` or `CompiledAction::Correct` and overwrites stored facts). With a deliberate break that disables the gate (`MemoryTurnGate::disabled()`), the small-talk and `#1718` anaphoric controls compile to `Assert`/`Correct` instead of `Unresolved` and fail the test.

## Files in scope

- `crates/uor-r4-training/**`
- `docs/compute/**`
- `docs/labs/m3-memory-turn-discriminator-gate/**`

## Done when

- `crates/uor-r4-training/src/relation_compiler.rs` implements the integer-only, squashing-free, matmul-free `MemoryTurnGate` (`StatementMargin`, `has_memory_turn_cue`, `gate_compiled_action`, `resolve_op_policy_gated`, and integration into `SavedCompiler::action` and `SavedCompiler::table_action`) that returns `CompiledAction::Unresolved` whenever a candidate `Assert`/`Correct` fails the margin or memory-turn discriminator check (never turning a write into a read), while preserving genuine `world=v2` and open-relation statements.
- Finite CPU controls in `crates/uor-r4-training/src/relation_compiler.rs` verify: (a) a hand-built compiler where genuine statements clear the margin and small-talk / opinion / non-relation questions / `#1718` anaphoric / third-person near-miss turns return `Unresolved`; (b) zero refusal of real `world=v2` relation statements; and (c) grading over the frozen 50-turn `data/panels/nonrelation-prose-v1.txt` panel (`7c29c665…`) via `grade_prose_panel` reduces false asserts and false overwrites to `0 / 50`.
- Design note posted on `#2855` and recorded in `docs/labs/m3-memory-turn-discriminator-gate/README.md`, with the one-page pre-registration sheet staged at `docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md` (`TEST FITNESS: PENDING`).

## Stop and ask if

- Any file outside the `ORDER.md` scope must change.
- Any GPU evaluation on `compiler-save-op-v25-rawtable` / `op-model-v25` or bar/spend registration is needed (awaits non-author `TEST FITNESS: FIT` and owner release on `#2855`).
