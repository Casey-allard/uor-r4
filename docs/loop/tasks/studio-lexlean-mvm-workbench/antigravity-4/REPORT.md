# REPORT for Decision D90 and Qwen3-0.6B Foundry Webharness LoRA pre-registration

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md) (#3939).

## What did not work

- Not a retry of a failed build or training run; this PR records owner decisions received at `2026-11-22T21:39:18Z` and pre-registers the Phase 2 GPU LoRA + `UORR4V13` compilation run (`PREP.md`) before renting any Vast.ai pod.
- Caught and resolved governance & pre-registration issues across two fresh-context non-author reviews before requesting merge:
  1. The interactive prompt referred to "D88" before checking [DECISIONS.md](../../../../integration/DECISIONS.md), where `D88` and `D89` were already assigned on `2026-11-16`; recorded cleanly as **D90** in the 6-column post-charter decision table (`Decisions after the charter (from D43)`).
  2. The pre-existing summary counter in [changes.md](../../../../rules/changes.md) was stale at `9` while `origin/main` already carried `20` dated `## ` entries; updated the counter accurately to `21` (entry `#21`).
  3. Initial `PREP.md` reviews (`8d05e94a`, `9371e10a`) flagged that upstream `gpu/recompile_holo_v13.rs` at `Casey-allard/lexlean-r4@e782e7bc` hardcodes `-arch=sm_89`, `12` OOD parity positions, `"float_ops": 0`, and lacks a CLI `val.jsonl` runner or `.pack` bundler, while the local scripts did not exist yet on disk. Resolved those items by writing and self-testing the local suite under `local/antigravity/d90-foundry-lora/`, and a follow-up check by `b0fe6f05` caught `E0308` (`HoloChatTurnMetrics::active_backend` `&'static str` vs `String` in `foundry_v13_eval_pack.rs`) plus vacuous/non-failing edge cases in `eval_foundry_v13.py`'s WAT scan and exact `--wasm`/bin paths in `PREP.md` — all fixed, self-tested, and re-pinned by SHA-256.

## What changed

- **[DECISIONS.md](../../../../integration/DECISIONS.md):** Added full 6-column table row `D90` (`2026-11-22`) under `Decisions after the charter (from D43)`, scoped strictly to `web/studio/` + `Casey-allard/lexlean-r4@weights-v13-qwen3-0.6b-foundry` (Charter `§3.2/§3.3/§3.4/§3.7/§3.10`), leaving `crates/**` and native M1–M8 training/serving quarantined under Charter §3.
- **[changes.md](../../../../rules/changes.md):** Added dated Charter change-log Entry `#21` (`22 November 2026`) quoting all three verbatim owner answers (`2026-11-22T21:39:18Z`) and updated the stale summary entry counter (`9` -> `21`).
- **[PLAN.md](../../../../../PLAN.md):** Updated the `2026-11-22` open owner-direction row (`studio-lexlean-mvm-workbench`) with the D90 two-stage overhaul sequence while keeping `src-tauri/` desktop work on hold.
- **[PREP.md](PREP.md):** Pre-registered the single-GPU (`--count 1`) Vast.ai `Qwen/Qwen3-0.6B` (`c1899de289a04d12100db370d81485cdf75e47ca`) Foundry webharness LoRA fine-tune (`DATA_SEED=20261122`, `TRAIN_SEED=42`), pinned local script/Rust harness SHA-256s (`foundry_schemas.json`, `generate_foundry_sft.py`, `train_foundry_lora.py`, `eval_foundry_v13.py`, `foundry_v13_eval_pack.rs`, `patch_recompile.py`), measured untuned baseline (`baseline_eval.json`), `wasm-tools` WAT float audit (`float_ops == 0`), `576`-position compiled `HoloV13Model` parity (`foundry_v13_parity.json`), `400`-episode compiled runtime eval (`foundry_v13_eval.json`), and `pack_XX.pack` XOR 2-of-3 verification (`foundry_pack_verify.json`).

## Measurements

- Pre-registration documentation & owner-decision PR only; baseline, BF16 LoRA, and compiled `UORR4V13` + Stitch pack measurements will be produced in Phase 2 after PR #4175 passes non-author `FIT` + exact-head review and merges to `main`.
- Verified local untracked harness self-test (`python3 local/antigravity/d90-foundry-lora/eval_foundry_v13.py --self-test`): generated `6,400` train (`sha256: 4fa37181...`) and `400` distinct `val` rows (`sha256: 8ae0ec5c...`, `train_val_overlap_hashes: 0`), with `320/320` (`100.0%`) gold tool/build validity and `80/80` (`100.0%`) gold Chat pass.

## Checks run at the head

- Non-author Charter §4.5 / D22 pre-registration fitness review (`bf39fabc-e1e5-4ce8-b071-e40c1c6e0ea9` at `4e1f9e118c0e1fdcedd2ff75edddf76a576fb908`): `FIT_VERDICT: FIT`, `VERDICT: APPROVE`.
- `uor-harness check --staged`: `0 rules, 0 failed (0 error, 0 warn)`.
- `cargo run -q --locked -p xtask -- links`: `0` broken links across PR #4175 changed files (`PLAN.md`, `DECISIONS.md`, `changes.md`, `ORDER.md`, `PREP.md`, `REPORT.md`).
- `cargo run -q --locked -p uor-r4-claim-wording -- .`: passed.
- `uor-harness ratchet d23` / `d25` / `d32`: passed (`4451` / `93` / `31`, unchanged from baseline).
- `uor-harness loop pr-check --base origin/main --head HEAD --local`: `verdict: PASS` (`PASS order`, `PASS banned-presence`, `PASS commit-order`, `PASS report`, `PASS owner-paths`, `PASS rule-sources`, `PASS rendered-lock`).

## Skills used

- None outside standard harness / git workflow.

## Still open

- Post `[antigravity] uor-review v1`, confirm exact-head review + CI green on final HEAD, merge PR #4175 via `uor-harness loop queue`, then execute Phase 2 GPU LoRA + `UORR4V13` pack publishing on Vast.ai under the `FIT`-cleared `PREP.md`.

## Plain summary

Recorded Owner Decision **D90** (`2026-11-22T21:39:18Z`) as row `D90` in `DECISIONS.md` and Entry `#21` in `changes.md` (correcting the stale `9` counter to `21`), and resolved all second-round pre-registration blockers by creating and self-testing the complete local SFT, LoRA, WASM WAT float-audit, compiled `HoloV13Model` `576`-position/`400`-episode evaluator, and `pack_XX.pack` XOR 2-of-3 verifier suite pinned by SHA-256 inside `PREP.md`.
