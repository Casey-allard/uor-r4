# REPORT: m6-reasoning-panel-freeze, antigravity-20

Lab: antigravity. Item: [m6-reasoning-panel-freeze](../../../../roadmap/items/m6-reasoning-panel-freeze.md). PR: #2977.

## What did not work, first

- **Initial assumption that an unconditioned numeric count-table baseline would score `0 / 240` across all 8 training seeds:** When we first ran `lawful_count_table_baseline_and_noise_measurement` across `8` independent seeded count-table draws, `numeric_only_scores` came back `[2, 1, 1, 2, 0, 0, 0, 0]` (`0..=2 / 240`) rather than `0 / 240`. Inspecting the collisions showed that the `20` `ModularStepCap` rows in `AlteredConstraints` reduce their final output modulo $M \in \{17, \dots, 53\}$, so a small remainder in `0..53` occasionally collides with the modal numeric value from a disjoint training draw (`1..=2` rows on 4 of the 8 seeds, while `Compositions` and `CodeTasks` score `0 / 80` across all 8 seeds). Rather than hiding or suppressing those collisions, we recorded the exact vector `[2, 1, 1, 2, 0, 0, 0, 0]` alongside the context-keyed count-table scores `[12, 16, 17, 18, 14, 17, 12, 18]` (min `12 / 240`, max `18 / 240`, half-band `+-3 rows`) and the family-conditioned modal ceiling (`20 / 240` from the `20` `ABSTAIN:UNSATISFIABLE` rows) in [`panel_m6_reasoning.rs`](../../../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs) and [`m6-reasoning.json`](../../../../harness/panels/m6-reasoning.json).
- **Closure borrow conflict in `SandboxProgram::execute` and Clippy lints:** The initial `check_reg` closure inside `SandboxProgram::execute` checked `r < regs.len()`, which borrowed `regs` immutably for the lifetime of the closure and prevented mutating `regs[dst]` (`error[E0506]`). Comparing against the fixed register file size (`r < 8`) resolved the borrow conflict. Additionally, CI `cargo clippy -p uor-r4-harness --locked --all-targets -- -D warnings` flagged `sub_rng.next_u64() % 2 == 0` (`clippy::manual_is_multiple_of`) and numbered doc-comment continuation indentation (`clippy::doc_lazy_continuation`); replacing with `.is_multiple_of(2)` and indenting the continuation lines resolved both.
- **Notebook scope boundary (`docs/research/notebook/m6-reasoning-coding.md`):** Step 7 of [`m6-reasoning-panel-freeze.md`](../../../../roadmap/items/m6-reasoning-panel-freeze.md) mentions `docs/research/notebook/m6-reasoning-coding.md`, which is outside the item's machine-checked `scope:` (`[crates/uor-r4-harness/src/panel*.rs, crates/uor-r4-harness/tests/panel*.rs, docs/harness/panels/**, docs/reference/storage.md, docs/labs/m6-reasoning-panel-freeze/**]`). To keep `loop pr-check` (`scope`) green, we recorded the six-column M6 notebook row in [`docs/labs/m6-reasoning-panel-freeze/README.md`](../../../../labs/m6-reasoning-panel-freeze/README.md).

## What changed

- [`crates/uor-r4-harness/src/panel_m6_reasoning.rs`](../../../../../crates/uor-r4-harness/src/panel_m6_reasoning.rs) and [`crates/uor-r4-harness/src/panels.rs`](../../../../../crates/uor-r4-harness/src/panels.rs): Built and exposed `uor_r4_harness::panels::m6_reasoning` providing:
  - Deterministic SplitMix64 synthesis (`generate_frozen_m6_panel`, `generate_m6_panel`) of `240` fresh rows (`80` `Compositions`, `80` `AlteredConstraints`, `80` `CodeTasks`) from canonical seed `M6_PANEL_FROZEN_SEED_V1 = 0x004D_365F_5245_4153`;
  - Deterministic D28-compliant 3-family oracles (`grade_m6_row`, `grade_m6_panel`, `grade_code_task_program`) including the 256-step zero-I/O `SandboxProgram` integer register machine and standalone Rust `#[test]` transpiler (`SandboxProgram::to_rust_source`);
  - Overlap exclusion (`M6ReasoningPanel::verify_zero_overlap`) against `FORBIDDEN_CONTROL_PAIRS` (`18` pairs) and `FORBIDDEN_CONTROL_TRIPLES` (`32` triples) from `m6_exact_arithmetic.rs` and `m6_ablation.rs`;
  - Lawful non-neural count-table baseline and noise evaluator (`M6CountTableBaseline`, `evaluate_count_table_baseline_noise`).
- [`crates/uor-r4-harness/tests/panel_m6_reasoning.rs`](../../../../../crates/uor-r4-harness/tests/panel_m6_reasoning.rs): Added 5 integration tests (`panel_seed_determinism_and_sha256_seal`, `panel_overlap_exclusion_and_corpus_disjointness`, `family_oracles_accept_correct_and_reject_wrong_answers`, `lawful_count_table_baseline_and_noise_measurement`, `m6_reasoning_json_registry_is_frozen_and_scorable`).
- [`docs/harness/panels/m6-reasoning.json`](../../../../harness/panels/m6-reasoning.json): Froze `m6-reasoning` (`"frozen": true`), set the deterministic D28 `grader`, recorded the measured `+-3 rows` count-table noise band, and registered the 4 bars (`overall-pass-240`, `compositions-80`, `altered-constraints-80`, `code-tasks-80`).
- [`docs/reference/storage.md`](../../../../reference/storage.md) and [`docs/labs/m6-reasoning-panel-freeze/README.md`](../../../../labs/m6-reasoning-panel-freeze/README.md): Recorded the unscored (`79,215 B`, `sha256:ac2034ed…`) and sealed oracle (`206,422 B`, `sha256:d1e5e4eb…`) JSONL content hashes, the six-column M6 notebook row, and the 8-block pre-registration / `TEST FITNESS: FIT` specification.
- [`docs/roadmap/items/m6-reasoning-panel-freeze.md`](../../../../roadmap/items/m6-reasoning-panel-freeze.md) and [`docs/roadmap/README.md`](../../../../roadmap/README.md): Marked `m6-reasoning-panel-freeze` `status: done` (closes #2866) and re-rendered the roadmap index.

## Measurements

- **Frozen panel size & SHA-256 content seal (`M6_PANEL_FROZEN_SEED_V1 = 0x004D365F52454153`):**
  - Total rows: `240` (`80` `Compositions`, `80` `AlteredConstraints`, `80` `CodeTasks`).
  - Unscored prompt JSONL stream: `79,215` bytes, SHA-256 `ac2034eda9b8ade5b6c760461a2c40c167955289a7f4a53c5ad1081894ab3edc`.
  - Sealed oracle JSONL stream: `206,422` bytes, SHA-256 `d1e5e4eb295d06d711b90e79805e6e35a84caa47666baeed6e202e203d407203`.
- **Oracle positive & negative controls:**
  - Canonical ground-truth responses: `240 / 240` pass (`80/80` `Compositions`, `80/80` `AlteredConstraints`, `80/80` `CodeTasks`).
  - Perturbed intermediate/final/habit-capture/vector responses: `0 / 240` pass (`60 / 60` satisfiable `AlteredConstraints` rows flagged as `HabitCaptureError`).
  - Constant `ABSTAIN:UNSATISFIABLE` across all 240 rows: `20 / 240` pass (`0/80` `Compositions`, `20/80` `AlteredConstraints` abstention slice, `0/60` satisfiable `AlteredConstraints`, `0/80` `CodeTasks`).
- **Lawful count-table baseline & noise (`8` independent seeded draws):**
  - Unconditioned numeric count table: `[2, 1, 1, 2, 0, 0, 0, 0]` (`0..=2 / 240`).
  - Context-keyed count table: `[12, 16, 17, 18, 14, 17, 12, 18]` (min `12 / 240`, max `18 / 240`, half-band `+-3 rows`).
  - Family-conditioned modal ceiling: `20 / 240` (`8.33%`).

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --test panel_m6_reasoning`: `5 passed; 0 failed`.
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: `88 items OK`.
- `uor-harness loop report --check` and `uor-harness loop pr-check --local`: `PASS`.
- `uor-harness loop gates --full`: `all 8 gates GREEN`.

## Skills used

- `uor-research-architect`, `solution-integrity`, `scope-guard`, `gates`.

## Still open

- Owner confirmation of the proposed M6 numeric bars (`overall-pass-240 >= 168/240`, `compositions-80 >= 56/80`, `altered-constraints-80 >= 56/80`, `code-tasks-80 >= 56/80` across `>= 2` seeds) posted on [`#2028` comment `6344266280`](https://github.com/UOR-Foundation/uor-r4/issues/2028#issuecomment-6344266280) (CHARTER §5.2).
- M6 GPU qualification (`m6-qualify-reasoning-coding` `#2865`) remains HELD pending M1 trunk training (`m1-trunk-assembly-and-training` `#2845`), PM dispatch, and Casey's typed D40 release.

## Plain summary

Built and froze the 240-row `m6-reasoning` acceptance panel (`80` unseen multi-step compositions, `80` altered constraints, `80` executable coding tasks) and its deterministic D28-compliant oracles in `crates/uor-r4-harness/src/panel_m6_reasoning.rs`, verified zero overlap against the CPU control literals and training corpora, measured a `+-3 rows` noise band (`12..=18 / 240`, modal ceiling `20 / 240`) with a lawful non-neural count-table baseline, and froze `docs/harness/panels/m6-reasoning.json` (closes #2866).
