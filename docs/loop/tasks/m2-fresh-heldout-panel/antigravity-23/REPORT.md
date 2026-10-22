# REPORT: m2-fresh-heldout-panel, antigravity-23

- Order: [`ORDER.md`](./ORDER.md)
- Branch: `antigravity/m2-fresh-heldout-panel`
- Head: `f703cf7903a865c8d28def5a367735c8ac1a1202`
- References #2849
- References #2030

## What did not work, first

- **Initial `docs/harness/panels/m2-fresh256.json` schema and 3-level relative links in `antigravity-22/REPORT.md`:** In `f67ac2525`, `docs/harness/panels/m2-fresh256.json` included extra top-level metadata keys (`title`, `status`, `schema`, `protocol`, `generator`, `storage_custody`, `overlap_exclusion`, `test_fitness`) and non-string `rows`/`seeds` values, failing `contract::tests::every_format_passes_on_the_real_state` (`docs/harness/state-formats.json::panel_shape`). In addition, `docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/REPORT.md` used `../../../` instead of `../../../../` for its 12 relative links, failing `cargo run --locked -p xtask -- links`. Both were fixed and verified against `every_format_passes_on_the_real_state` and `xtask links`.
- **Single-row scoring of question-blind controls (`copy_first_stated = 120/256`, `copy_last_stated = 128/256`) above the `103/256` bar:** Because each history has 2 opposite-role queries (`job` and `home`), a question-blind heuristic that always copies turn 0 or always copies the last assertion turn gets 1 of the 2 queries right per history (`120/256` or `128/256` single rows) while answering both `job?` and `home?` identically. Wiring `"pair_id"` on every case in `labels.json` (`pairs_both_complete_distinct` in [`crates/uor-r4-core/examples/native-bank-generalization.rs`](../../../../../crates/uor-r4-core/examples/native-bank-generalization.rs)) requires both opposite-role queries of a history to be answered completely and distinctly, reducing all 4 question-blind and role-swap controls (`copy_first_stated`, `copy_last_stated`, `role_swap_distractor`, `best_constant_reply`) to `0 / 256` pair-verified rows (`0 / 128` histories, strictly `< 103`).
- **Rare multi-subword words in `FRESH256_LEN8` exceeding `MAX_TOTAL_REPLAY_TOKENS = 128` under the real `ByteBpeTokenizer` (`d36d3e87…`):** Switching from placeholder word-index IDs to the exact pinned `ByteBpeTokenizer` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`) split rare words (`astrolabe`, `aqueduct`, `hourglass`, `vestibule`, etc.) into 3–4 subword tokens each, pushing `query.len() + replay_len + target.len()` past `128`. Replacing those with 1–2 token words in `FRESH256_LEN8` brought `max_total_replay_tokens` to `105 <= 128` and `max_target_tokens` to `19 <= 32` while maintaining `0` overlaps against all `74` historical literals and `0` intra-panel duplicate content words across all `34` fresh literals (`107` distinct content words).
- **Marking `m2-fresh-heldout-panel` `done` and `frozen: true` before external non-author FIT review and HF store upload:** Because a subagent spawned by the authoring session shares the author's session framing and the 4 sealed files are staged locally awaiting maintainer upload to `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/`, `docs/harness/panels/m2-fresh256.json` is set to `"frozen": false` (`TEST FITNESS: PENDING`), `docs/roadmap/items/m2-fresh-heldout-panel.md` remains `status: ready`, and this PR uses `References #2849`.

## What changed

- [`docs/labs/m2-fresh-heldout-panel/instrument/src/lib.rs`](../../../../labs/m2-fresh-heldout-panel/instrument/src/lib.rs) and [`src/main.rs`](../../../../labs/m2-fresh-heldout-panel/instrument/src/main.rs):
  - Parses the real repository files on disk ([`crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs`](../../../../../crates/uor-r4-training/examples/geometric_native_compiler/curriculum.rs) for all `64` curriculum literals and [`crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs`](../../../../../crates/uor-r4-training/examples/geometric-native-bank-observation-plan.rs) for the `10` untouched transfer literals = `74` historical literals total, plus [`archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json`](../../../../../archive/labs/m2-fresh-437-2026-10-10/fresh-preparation-audit-attempt1.json) and [`archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json`](../../../../../archive/labs/m2-fresh-437-2026-10-10/development-replay-audit-summary.json)).
  - Replaces `"granitecauseway annex"` with `"syenite annex"`, `"slate conservatory"` with `"chert conservatory"`, and updates `FRESH256_LEN8` so all `34` fresh literals (`107` content words) have `0` intra-panel duplicate content words and `0` exact/word/2-gram overlaps against all `74` historical literals.
  - Encodes every utterance, literal, target, and query with the exact pinned `ByteBpeTokenizer` (`d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`), emits `StackStore` (`record = commit = event + 1`), validates `inputs.json` and `labels.json` against `#[serde(deny_unknown_fields)]` consumer structs (`ProbeInputs`, `ProbeLabels`) with `"pair_id"` on every case, verifies all 4 controls score `0 / 256` (`< 103`) under history-pair completion, and adds unit test `fresh256_panel_hashes_match_registered_panel_json`.
- [`docs/harness/panels/m2-fresh256.json`](../../../../harness/panels/m2-fresh256.json) and [`docs/harness/panels/m2-reply512.json`](../../../../harness/panels/m2-reply512.json): updated `m2-fresh256.json` to conform strictly to `panel_shape` in `docs/harness/state-formats.json` (`frozen: false`, `TEST FITNESS: PENDING`, updated SHA-256 digests) and clarified single-model (`saved437`) within-panel spread vs `unstated` multi-seed training spread in both panel files.
- [`docs/labs/m2-fresh-heldout-panel/README.md`](../../../../labs/m2-fresh-heldout-panel/README.md), [`docs/reference/storage.md`](../../../../reference/storage.md), [`docs/mechanisms/native-geometric.md`](../../../../mechanisms/native-geometric.md), [`docs/loop/tasks/m2-fresh-heldout-panel/antigravity-22/REPORT.md`](../antigravity-22/REPORT.md), [`docs/roadmap/items/m2-fresh-heldout-panel.md`](../../../../roadmap/items/m2-fresh-heldout-panel.md), and [`docs/roadmap/README.md`](../../../../roadmap/README.md): fixed the 12 relative links in `antigravity-22/REPORT.md`, updated all paths, counts (`74` historical literals), token budgets, control scores, and SHA-256 digests, and restored `m2-fresh-heldout-panel.md` to `status: ready`.

## Measurements

- **Real-file historical overlap and intra-panel disjointness (`verify_m2_fresh256_panel`):**
  - Historical literals parsed from repository source files on disk: `74` (`64` in `curriculum.rs` + `10` untouched transfer literals in `geometric-native-bank-observation-plan.rs`).
  - Overlaps against all `74` historical literals: `0` exact, `0` content-word, `0` word 2-gram; minimum same-stratum character Levenshtein edit distance `10 >= 8`.
  - Intra-panel content-word duplicates across all `34` fresh literals (`107` distinct content words): `0`.
- **Exact `ByteBpeTokenizer` (`d36d3e87…`) budgets and history-pair control verification:**
  - `max_turn_tokens = 26` (`<= 128`), `max_target_tokens = 19` (`<= 32`), `max_total_replay_tokens = 105` (`<= 128`).
  - History-pair completion (`pair_id` in `labels.json`, `pairs_both_complete_distinct` in `native-bank-generalization.rs`): `copy_first_stated = 0 / 256` (`0 / 128` histories), `copy_last_stated = 0 / 256` (`0 / 128` histories), `role_swap_distractor = 0 / 256`, `best_constant_reply = 0 / 256` (all `< 103`).
- **Updated sealed artifacts staged at `local/antigravity/m2-fresh256-sealed/`:**
  - `plan.json`: `122,268 B`, `sha256:0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715`
  - `inputs.json`: `524,850 B`, `sha256:1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a` (`0` occurrences of `"accepted"`)
  - `labels.json`: `68,000 B`, `sha256:3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be`
  - `manifest.json`: `1,273 B`, `sha256:c930e1f0275ea0fc8c8b2afbb6062546ff11331c1b7fdfeb9ec75eca97e456bd`

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo fmt --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --check`: exit `0`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo clippy --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked --all-targets -- -D warnings`: exit `0`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked`: `4 passed; 0 failed` (`dev512_spread_matches_sealed_saved437_receipts`, `fresh256_panel_hashes_match_registered_panel_json`, `fresh256_panel_passes_all_overlap_balance_and_seal_checks`, `negative_mutations_are_rejected_by_verifier`)
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib contract::tests::every_format_passes_on_the_real_state`: `1 passed; 0 failed`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- links`: `0` broken links in changed files
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- roadmap check`: `88 items OK`
- `uor-harness loop gates --full`: `GREEN` across all 12 gates
- `uor-harness loop report --check`: `PASS`
- `uor-harness loop pr-check --local`: `verdict: PASS`

## Skills used

- `uor-research-architect`, `scope-guard`, `solution-integrity`, `gates`, `context-guard`

## Still open

- Independent non-author lab `TEST FITNESS: FIT` review of `m2-fresh256` (`#2849` step 5) and flipping `frozen: true` once reviewed.
- Maintainer upload of `plan.json`, `inputs.json`, `labels.json`, and `manifest.json` from `local/antigravity/m2-fresh256-sealed/` to `hf://datasets/caseyallard/uor-r4-store/data/m2-fresh256/` (`#2849` step 4).
- Copying the six-column M2 notebook row from `docs/labs/m2-fresh-heldout-panel/README.md` (`§6`) into `docs/research/notebook/m2-grounded-reply.md` once an item scope includes that path.

## Plain summary

Repaired PR `#2985` (`antigravity-23`): fixed the 12 relative links in `antigravity-22/REPORT.md` and formatted `docs/harness/panels/m2-fresh256.json` to pass `contract::tests::every_format_passes_on_the_real_state`, wired `verify_m2_fresh256_panel` to parse the actual repository files on disk (`74` historical literals, `0` overlaps, `0` intra-panel duplicate content words), encoded all panel artifacts with the exact pinned `ByteBpeTokenizer` (`d36d3e87…`) and consumer-compatible `inputs.json`/`labels.json` schemas (`pair_id` on every case so all 4 question-blind and role-swap controls score `0 / 256 < 103`), and kept `m2-fresh-heldout-panel` `ready` (`frozen: false`, `TEST FITNESS: PENDING`) pending an external lab FIT review and HF store upload.
