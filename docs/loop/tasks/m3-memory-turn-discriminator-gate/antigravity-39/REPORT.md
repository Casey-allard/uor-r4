# REPORT: m3-memory-turn-discriminator-gate, antigravity-39

Lab: antigravity. Order: [ORDER.md](ORDER.md).

## What did not work, first

- **Initial open-relation branch in `MemoryTurnGate::check_turn_structure` accepted a non-copular possessive turn when swept against a fixed derived relation ID (`derived_relation_id("sculptor")`):** In the first draft of `check_turn_structure`, the open-relation arm checked `relation_phrase(source).is_some()` and a first-person cue without verifying that `derived_relation_id(&phrase) == relation` and that the copula `"is"` immediately precedes the candidate value span. Because `relation_phrase` scans to the end of the turn when no `"is"` occurs after `"my"`, one row in `nonrelation-prose-v1.txt` slipped through to `grade_prose_panel`, which rejected the derived ID (`>= DERIVED_RELATION_ID_BASE`) as outside the 10 closed `world=v2` labels. Tightening the open-relation arm in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1838-L1854) to require `derived_relation_id(&phrase) == relation` and `prefix_words.last() == Some("is")` resolved this cleanly (`0 / 50` false writes across all `22` closed and derived relation sweeps).
- **Hyphenated cue `"go-to"` in `favorite_food`:** `word_spans("My go-to dish is risotto.")` splits on `'-'` into `"go"` and `"to"`, so checking `has("go-to")` initially rejected that template. Updating the `favorite_food` frame rule to `has("dish") && has("my")` (with third-person possessors like `"my sister's"` already excluded by `has_third_person_possessor`) preserved all `122 / 122` closed templates across `world=v1` and `world=v2`.
- **Broken link in initial lab docs (`../../mechanisms/exact-memory-store.md`):** `xtask links` flagged `../../mechanisms/exact-memory-store.md` in `README.md` and `preregistration-DRAFT-2026-10-23.md`; updated both to the canonical mechanism page [`../../mechanisms/exact-memory.md`](../../../../mechanisms/exact-memory.md).
- **What this PR does not close:** GPU evaluation of the gated `world=v2` session recipe (`compiler-save-op-v25-rawtable` + `op-model-v25`, `op_policy = unless_query`) on [`data/panels/nonrelation-prose-v1.txt`](../../../../../data/panels/nonrelation-prose-v1.txt) and across the 3 held-out seeds (`21, 22, 23`) of 500 `world=v2` dialogues (`1,500` dialogues / `7,605` turns) is a GPU qualification run (CHARTER §4.5) that awaits a non-author `TEST FITNESS: FIT` review of [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md) and owner/PM registration on `#2855`.

## What changed

- **Code (`+433 / -11` lines in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1608-L2039)):** Added [`StatementMargin`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1614-L1664), [`canonical_relation_name`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1667-L1682), [`MemoryTurnGate`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1690-L1856) (including `MemoryTurnGate::disabled()`, `check_margin`, and `check_turn_structure`), [`has_memory_turn_cue`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1964-L1973), [`gate_compiled_action`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L1980-L2010), and [`resolve_op_policy_gated`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L2015-L2039), and wired `MemoryTurnGate::default()` into `SavedCompiler::action` and `SavedCompiler::table_action`.
- **Tests (`+583` lines in [`crates/uor-r4-training/src/relation_compiler.rs`](../../../../../crates/uor-r4-training/src/relation_compiler.rs#L4224-L4806)):** Added 3 finite CPU unit tests (`memory_turn_discriminator_and_margin_gate_controls`, `memory_turn_gate_preserves_all_world_v2_relation_statement_templates`, and `memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`).
- **Docs (`+235` lines across 3 files):** Added [`docs/labs/m3-memory-turn-discriminator-gate/README.md`](../../../../labs/m3-memory-turn-discriminator-gate/README.md), [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md), and the task order/report (`1.0x` file-scope ratio against [`ORDER.md`](ORDER.md)).

## Measurements

- **Closed and open relation statement template preservation (`memory_turn_gate_preserves_all_world_v2_relation_statement_templates`):**
  - Command: `~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler::tests::memory_turn_gate_preserves_all_world_v2_relation_statement_templates`
  - Result: `122 / 122` closed-relation `assert` and `update` templates across all `10` relations in `world=v1` and `world=v2` plus `3 / 3` open-relation templates (`>= DERIVED_RELATION_ID_BASE`) preserved (`0` false rejections).
- **Frozen 50-row non-relation prose panel adversarial sweep (`memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`):**
  - Command: `~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler::tests::memory_turn_gate_achieves_zero_false_writes_on_frozen_nonrelation_prose_panel`
  - Conditions: All `50` frozen rows across all `6` strata of [`data/panels/nonrelation-prose-v1.txt`](../../../../../data/panels/nonrelation-prose-v1.txt) (`5,111 B`, `sha256: 7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7`) graded via `grade_prose_panel(&panel, &compiler, true, true)` across `11` relation IDs (`1..=10` and `derived_relation_id("sculptor")`) $\times$ `2` write acts (`Assert`, `Correct`) = `22` sweeps (`1,100` candidate writes).
  - Result: `clean_unresolved_count = 50 / 50`, `false_assert_count = 0 / 50`, `false_correct_action_count = 0 / 50`, `false_overwrite_count = 0 / 50`, `false_query_count = 0 / 50` across all `22` sweeps.

## Checks run at the head

- `~/.cargo/bin/cargo test -p uor-r4-training --lib relation_compiler`: `26 passed; 0 failed`
- `~/.cargo/bin/cargo test -p uor-r4-training --lib nonrelation_prose_panel`: `5 passed; 0 failed`
- `uor-harness ratchet d23`: `OK: no softmax added. 4451 remaining across 165 files (baseline 4451).`
- `uor-harness ratchet d25`: `OK: no dense matrix product call sites added. 93 remaining across 26 files (baseline 93).`
- `uor-harness ratchet d32`: `OK: no discrete-path floating-point sites added. 31 remaining across 1 files (baseline 31).`
- `uor-harness loop report --check`: `PASS`
- `uor-harness loop pr-check --local`: `PASS`
- `uor-harness loop gates --full`: `PASS`

## Skills used

- [`uor-research-architect`](../../../../../.agents/skills/uor-research-architect/SKILL.md): used to design the integer/additive margin and structural memory-turn discriminator gate in `relation_compiler.rs` with zero squashing/matmul/float ratchets and stage the `DRAFT` pre-registration sheet.

## Still open

- Non-author `TEST FITNESS: FIT` review of [`docs/labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md`](../../../../labs/m3-memory-turn-discriminator-gate/preregistration-DRAFT-2026-10-23.md), owner/PM registration of the bar and `<= $0.15` pod spend on `#2855`, and the two-arm GPU evaluation on `compiler-save-op-v25-rawtable` + `op-model-v25`. [`docs/roadmap/items/m3-memory-turn-discriminator-gate.md`](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md) remains `status: ready`. References #2855, #2031.

## Plain summary

In Milestone M3 (Exact Memory & Grounded Sessions), the turn compiler reads each user sentence in a conversation and decides whether to store a new fact (`Assert`), update an existing fact (`Correct`), read back a stored fact (`QueryCurrent` / `Query`), or leave memory untouched (`Unresolved`). In earlier runs (`#1718`), ordinary conversational sentences that did not state any personal fact (such as `"The weather has been unusually warm this week."` or `"Actually, let us set that topic aside for another conversation."`) were sometimes misclassified by the auxiliary operation model (`op-model-v25`) as memory writes under the `unless_query` policy, silently overwriting stored facts. This task adds an integer-only, squashing-free, matrix-multiplication-free gate (`StatementMargin` and `MemoryTurnGate`) at the compiler's write boundary in `crates/uor-r4-training/src/relation_compiler.rs`. Before any memory write is allowed, the gate checks both additive integer score gaps and that the words outside the value span form a non-interrogative, speaker-grounded statement for the target relation; any rejected write becomes `Unresolved` (and is never turned into a memory read). Finite CPU tests verify `122 / 122` closed-relation statement templates and `3 / 3` open-relation templates pass with `0` false rejections, and `0 / 50` false writes occur across `1,100` adversarial write probes on the frozen 50-row non-relation prose panel.

---

Task status: `status: ready` on [`m3-memory-turn-discriminator-gate`](../../../../roadmap/items/m3-memory-turn-discriminator-gate.md) (design, CPU implementation, finite controls, and `DRAFT` pre-registration sheet complete; GPU evaluation awaits non-author `FIT` and owner release on `#2855`).
