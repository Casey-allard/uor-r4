# ORDER for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v3`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `517379e47`. Updates the pre-registration sheet (`PREP.md`, Run `d90-foundry-lora-v3`) following Run 2 diagnostics on `vast-62057972` (`~$0.10` spent, cleanly terminated with `0` pods running; combined D90 spend `~$0.34` of `$3.00` cap).

## Cancel probe

Verify `uor-harness loop pr-check` (`PASS order`, `PASS commit-order`, `PASS report`), `xtask links`, `uor-r4-claim-wording`, ratchets `d23`/`d25`/`d32` (`0` added), zero binaries (§3.13), and non-author `FIT_VERDICT: FIT` + `VERDICT: APPROVE` on `antigravity-6/PREP.md` before renting Pod #3.

## Scope

Inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-6/**` only (zero `owner-paths` or `crates/**` modifications):
- `PREP.md`: records Run 2 empirical diagnostics (`wasm_float_audit` PASSED `passed: true` / `compute_float_ops == 0` / `js_abi_bridge_casts == 1`, `lora_eval` Bar #2 PASSED `96.88%` (`310/320`), Bar #3 Chat `87.50%` (`70/80`) isolated to `<tool_call>`/`<tool_response>` tag quoting in `CHAT_TOPICS_TRAIN[16]` + `18.75%` Chat share + `1`-epoch under-exposure on 4 Chat topics, plus `10` unseen `github_pull`/`github_push` verbs), pins updated SHA-256 digests (`generate_foundry_sft.py`, `train_foundry_lora.py`), keeps held-out `val.jsonl` (`8ae0ec5c...`) bit-identical, configures `2` epochs `lr=1.0e-4` (`dropout=0.08`, `wd=0.03`, best-`val_loss` adapter restore), Rust `1.97.1`, and caps Run 3 spend at `<= $2.00` / `2.0h` (combined `< $2.50` <= `$3.00` D90 cap).
- `REPORT.md`: summarizes Run 2 empirical findings and Run 3 pre-registration checks.

## Done check

- `uor-harness loop pr-check --base origin/main --head HEAD --local`, `uor-harness check --staged`, `xtask links`, `uor-r4-claim-wording`, ratchets `d23/d25/d32`, non-author `FIT` + `APPROVE`, and GitHub CI all pass.

## Stop and decide if

Stop if non-author reviewer finds `PREP.md` `NOT_FIT` or any file outside `antigravity-6/**` is modified.
