# REPORT: m2-fresh-heldout-panel, antigravity-24

- Order: [`ORDER.md`](./ORDER.md)
- Branch: `antigravity/m2-fresh-heldout-panel`
- Head: `5cb2409496bcfd131c1c65e8df3c5a42d4abd07c`
- References #2849
- References #2030

## What did not work, first

- **Mismatch between the single-row `target: 103` (`of: 256`) bar in `m2-fresh256.json` and the `pairs_both_complete_distinct` control metric:** In `antigravity-23`, `m2-fresh256.json` registered `target: 103` (`of: 256` single rows) while scoring `copy_first_stated` (`120/256` single rows) and `copy_last_stated` (`128/256` single rows) at `0/256` under `pairs_both_complete_distinct`. Per review `6349065628`, we stopped and asked the owner in chat; Casey approved registering `pairs_both_complete_distinct >= 52/128` (`40.625%` of the 128 opposite-role history pairs, with single-row `complete >= 104/256`) as the official M2 fresh-panel bar metric on [`#2030`](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6350135678).
- **Standalone string-comparison control check and unverified `PRETOKEN_BPE_TABLE` in `instrument/src/lib.rs`:** In `antigravity-23`, `verify_m2_fresh256_panel` checked the 4 controls by comparing `first_lit` / `last_lit` strings in `lib.rs` instead of running the real `FrozenAnswers` + `ByteBpeTokenizer` + `pair_id` grading loop from [`crates/uor-r4-core/examples/native-bank-generalization.rs`](../../../../../crates/uor-r4-core/examples/native-bank-generalization.rs), and `PRETOKEN_BPE_TABLE` was not cross-checked against the real `ByteBpeTokenizer` loaded from disk. Linking `uor-r4-tokenizer` and `uor-r4-core` into `docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml` resolved both.
- **Literal `complete_rows: 437` and unlabeled saved replay-audit fields in `analyze_saved437_dev512_spread`:** `analyze_saved437_dev512_spread` previously set `complete_rows: 437` as a literal and named `deterministic_replay_diff_rows: 0` without clarifying that the `0/512` replay audit (`PASS_SAVED_EVIDENCE`, `7,908` checked steps) is copied from the saved 2026-10-10 audit (`development-replay-audit-summary.json`), whereas the `219/256` vs `218/256` half-panel split and `6.24`-row / `8.64`-row 16-block cluster SE are computed from `.endpoints.final.complete_ids` in `reviewer-rank177.json`.

## What changed

- [`docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml`](../../../../labs/m2-fresh-heldout-panel/instrument/Cargo.toml) and [`src/lib.rs`](../../../../labs/m2-fresh-heldout-panel/instrument/src/lib.rs):
  - Linked `uor-r4-tokenizer` and `uor-r4-core` (`uor_r4_core::native_geometric::FrozenAnswers`).
  - Added `load_pinned_byte_bpe_tokenizer()` loading the pinned `tokenizer.json` (`sha256:d36d3e8700a123e620012df77de195f244fbdb4d05d9e1aa7e77fa9407590f89`) via `ByteBpeTokenizer::from_tokenizer_json_bytes` and verifying every entry of `PRETOKEN_BPE_TABLE` plus every panel turn, literal, query, and target token slice against `ByteBpeTokenizer::encode`.
  - Added `grade_generated_replies_with_evaluator_oracle()`, executing the exact `FrozenAnswers::validate` + `ByteBpeTokenizer::decode_bytes` + `FrozenAnswers::accepts` + `pair_groups` grading loop from `crates/uor-r4-core/examples/native-bank-generalization.rs:827-948` over deserialized `inputs.json` and `labels.json`, and verified that all 4 controls (`copy_first_stated`, `copy_last_stated`, `role_swap_distractor`, `best_constant_reply`) score `pairs_both_complete_distinct == 0` (`0 / 128` pairs, `0 / 256` pair-verified rows) while oracle ground-truth replies score `256 / 256` rows and `128 / 128` distinct pairs.
  - Updated `analyze_saved437_dev512_spread()` to set `complete_rows: complete_set.len()`, renamed the saved replay-audit fields to `saved_audit_status` (`"PASS_SAVED_EVIDENCE"`) and `saved_audit_checked_steps` (`7908`), and removed the redundant `!= 0` check on the control failure branch.
  - Updated `manifest.json` (`1,384 B`, `sha256:7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964`) with `"target_bar_metric": "pairs_both_complete_distinct"`, `"target_bar_pairs": 52`, `"target_bar_pairs_of": 128`, `"target_bar_rows": 104`, `"target_bar_of": 256`.
- [`docs/harness/panels/m2-fresh256.json`](../../../../harness/panels/m2-fresh256.json), [`docs/harness/panels/m2-reply512.json`](../../../../harness/panels/m2-reply512.json), [`docs/labs/m2-fresh-heldout-panel/README.md`](../../../../labs/m2-fresh-heldout-panel/README.md), [`docs/mechanisms/native-geometric.md`](../../../../mechanisms/native-geometric.md), [`docs/reference/storage.md`](../../../../reference/storage.md), and [`docs/loop/tasks/m2-fresh-heldout-panel/antigravity-23/REPORT.md`](../antigravity-23/REPORT.md):
  - Updated the `fresh-transfer-256` bar in `m2-fresh256.json` to `pairs_both_complete_distinct` (`target: 52`, `of: 128`, `complete >= 104/256`) per owner registration [`6350135678`](https://github.com/UOR-Foundation/uor-r4/issues/2030#issuecomment-6350135678) on `#2030`.
  - Updated all `manifest.json` byte/SHA-256 references (`1,384 B`, `sha256:7c33f3f7…`), labeled the saved 2026-10-10 replay audit vs computed `reviewer-rank177.json` spread in both panel files and `README.md`, and updated `Head:` in `antigravity-23/REPORT.md` to `f703cf7903a865c8d28def5a367735c8ac1a1202`.

## Measurements

- **Real `ByteBpeTokenizer` (`d36d3e87…`) + `FrozenAnswers` + `pair_id` grader verification (`real_byte_bpe_tokenizer_and_evaluator_oracle_verify_tokens_and_controls`):**
  - `PRETOKEN_BPE_TABLE` (`191` entries) and all `256` cases' turn, literal, query, and target token slices match `ByteBpeTokenizer::encode` 100% (`0` mismatches).
  - Oracle ground-truth answers: `complete = 256 / 256`, `pairs_both_complete_distinct = 128 / 128`.
  - Question-blind and role-swap controls through `grade_generated_replies_with_evaluator_oracle`: `copy_first_stated = 0 / 128` pairs (`complete = 120 / 256`), `copy_last_stated = 0 / 128` pairs (`complete = 128 / 256`), `role_swap_distractor = 0 / 128` pairs (`complete = 0 / 256`), `best_constant_reply = 0 / 128` pairs (`complete = 16 / 256`) — all `0 / 128 < 52 / 128`.
- **Sealed artifacts staged at `local/antigravity/m2-fresh256-sealed/`:**
  - `plan.json`: `122,268 B`, `sha256:0ed1ab98ab50bbad6c484c4e7e4ffeeb8bb7ad17d162251826302efe86d75715`
  - `inputs.json`: `524,850 B`, `sha256:1de2fc28607eff6bc412eee41a259d806ee1479b7c925059f00e9d3b54afa99a`
  - `labels.json`: `68,000 B`, `sha256:3b5c54771b8dc62e3c30aef2b30de28c09dfe79625f333854af046e6e354e2be`
  - `manifest.json`: `1,384 B`, `sha256:7c33f3f7e19edcb4b1a7aa5e5eab3bb0056db9351067d809f62f97b4a8880964`

## Checks run at the head

- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo fmt --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --check`: exit `0`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo clippy --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked --no-deps --all-targets -- -D warnings`: exit `0`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --manifest-path docs/labs/m2-fresh-heldout-panel/instrument/Cargo.toml --locked`: `5 passed; 0 failed` (`dev512_spread_matches_sealed_saved437_receipts`, `fresh256_panel_hashes_match_registered_panel_json`, `real_byte_bpe_tokenizer_and_evaluator_oracle_verify_tokens_and_controls`, `negative_mutations_are_rejected_by_verifier`, `fresh256_panel_passes_all_overlap_balance_and_seal_checks`)
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo test --locked -p uor-r4-harness --lib contract::tests::every_format_passes_on_the_real_state`: `1 passed; 0 failed`
- `CARGO_TARGET_DIR=/Users/casey.allard/uor-r4/local/harness-target ~/.cargo/bin/cargo run --locked -p xtask -- links`: `0` broken links in changed files
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

Repaired all HIGH, MEDIUM, and LOW findings from review `6349065628` on PR `#2985` (`antigravity-24`): aligned `m2-fresh256.json`, `manifest.json`, `README.md`, and `native-geometric.md` with the owner-registered bar on `#2030` (`6350135678`: `pairs_both_complete_distinct >= 52/128`, `complete >= 104/256`), linked `uor-r4-tokenizer` and `uor-r4-core` into the panel instrument to verify `PRETOKEN_BPE_TABLE` and all panel token slices against the real `ByteBpeTokenizer` (`d36d3e87…`) and score all 4 controls (`0/128` pairs) through the real `FrozenAnswers` + `ByteBpeTokenizer` + `pair_id` grader, computed `complete_rows = complete_set.len()` from `reviewer-rank177.json` while labeling the saved 2026-10-10 replay audit fields, and cleaned up the redundant `!= 0` check and `antigravity-23/REPORT.md` head SHA.
