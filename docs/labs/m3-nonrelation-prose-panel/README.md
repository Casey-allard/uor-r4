# Lab Record: `m3-nonrelation-prose-panel` (`#2856`, `antigravity-10`)

- **Item:** [`docs/roadmap/items/m3-nonrelation-prose-panel.md`](../../roadmap/items/m3-nonrelation-prose-panel.md) (`#2856`, folds `#1718`)
- **Design note:** `https://github.com/UOR-Foundation/uor-r4/issues/2856#issuecomment-6337042540`
- **Owner pass-bar decision card:** `https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6337042887`
- **Pre-registration sheet:** [`preregistration-DRAFT-2026-10-22.md`](preregistration-DRAFT-2026-10-22.md) (`TEST FITNESS: PENDING`)

## Deliverables landed in `antigravity-10`

1. **Frozen plain-text non-relation prose panel (`data/panels/nonrelation-prose-v1.txt`)**:
   - SHA-256: `7c29c665f97bae126db0cf82cee1795d2160c2ad35f852f64fffc92b836a3de7` (registered in [`data/panels/MANIFEST.sha256`](../../../data/panels/MANIFEST.sha256)).
   - 50 turns (`10` per category across `small_talk`, `opinion`, `nonrelation_question`, `anaphoric`, and `statement_near_miss`; `25` in `in_domain` and `25` in `heldout_v2_disjoint`).
   - The 25 `heldout_v2_disjoint` turns share zero 4-word consecutive runs with `crates/uor-r4-training/src/milestone_world.rs` and `crates/uor-r4-training/src/milestone_world_v2.rs`.
2. **Deterministic Rust grader (`crates/uor-r4-training/src/nonrelation_prose_panel.rs`)**:
   - Loads and SHA-256-verifies `data/panels/nonrelation-prose-v1.txt` (`ProsePanel::load_frozen`).
   - Seeds all 10 `world=v2` relations (`1..=10`: `user_name`, `pet_name`, `friend_name`, `hometown`, `lucky_number`, `code_word`, `job`, `home`, `favorite_food`, `favorite_color`) in a `StackStore` (`seed_world_v2_store`) and grades `TurnCompiler::compile` and `apply_prose_action` (`grade_prose_panel`).
   - Reports exact integer counts and `(numerator, denominator)` ratios for `clean_unresolved_count`, `false_assert_count`, `false_overwrite_count`, `false_correct_action_count`, and `false_query_count` overall, per category, and per split.
3. **Finite CPU test suite (`crates/uor-r4-training/src/nonrelation_prose_panel.rs::tests`)**:
   - `cargo test -p uor-r4-training --lib`: 42 passed (38 existing + 4 new in `nonrelation_prose_panel::tests`), 0 failed.
   - `cargo test -p uor-r4-training`: 67 passed total across 3 test binaries (`lib`: 42, `dialogue-recall-corpus` bin: 25, `doc-tests`: 0).
   - `uor-harness ratchet d23`: GREEN (`4451` remaining across `165` files, baseline `4451`, `+0`).
   - `uor-harness ratchet d25`: GREEN (`93` remaining across `26` files, baseline `93`, `+0`).

## Open items before closing `#2856`

- Non-author FIT review of [`preregistration-DRAFT-2026-10-22.md`](preregistration-DRAFT-2026-10-22.md), owner decision on the proposed pass bar (`#2028`), and typed owner release (CHARTER §4.5, §5.1, §5.2) to run the saved `world=v2` compiler over `data/panels/nonrelation-prose-v1.txt` on a leased GPU and record the baseline `false_assert` and `false_overwrite` rates in `docs/research/notebook/m3-durable-memory.md`.
