# ORDER for Qwen3-0.6B Foundry Webharness LoRA Addendum (`d90-foundry-lora-v2`)

Lab: antigravity. Item: [studio-lexlean-mvm-workbench](../../../../roadmap/items/studio-lexlean-mvm-workbench.md), #3939. Base: `c0ce2b3a7`. Updates the pre-registration sheet (`PREP.md`, Run `d90-foundry-lora-v2`) following Run 1 diagnostics on `vast-62052088` (`~$0.23` spent, cleanly terminated with `0` pods running).

## Cancel probe

Verify `uor-harness loop pr-check` (`PASS order`, `PASS commit-order`, `PASS report`), `xtask links`, `uor-r4-claim-wording`, ratchets `d23`/`d25`/`d32` (`0` added), zero binaries (§3.13), and non-author `FIT_VERDICT: FIT` + `VERDICT: APPROVE` on `antigravity-5/PREP.md` before renting Pod #2.

## Scope

Inside `docs/loop/tasks/studio-lexlean-mvm-workbench/antigravity-5/**` only (zero `owner-paths` or `crates/**` modifications):
- `PREP.md`: records Run 1 empirical negatives cleanly (`baseline_eval` `11.56%` tool / `87.50%` chat; 3-epoch `val_loss` rise stop rule; 1-epoch `lr=1.5e-4` `95.62%` tool / `82.50%` chat isolated to `TRAIN_CHAT_ANSWERS` 18-token fallback boilerplate on 2 underrepresented Chat topics + single `js_sys::Function::call2` `JsValue::from_f64` ABI cast in `generate_chat_turn_with_history`), pins all updated local script SHA-256 digests (`generate_foundry_sft.py`, `train_foundry_lora.py`, `eval_foundry_v13.py`), configures `1`-epoch `lr=1.2e-4` (`dropout=0.10`, `wd=0.05`) + `wasm_bindgen`/`js_sys` ABI cast separation (`compute_float_ops == 0`), and caps Run 2 spend at `<= $2.50` / `2h`.
- `REPORT.md`: summarizes Run 1 empirical findings and Run 2 pre-registration checks.

## Done check

- `uor-harness loop pr-check --base origin/main --head HEAD --local`, `uor-harness check --staged`, `xtask links`, `uor-r4-claim-wording`, ratchets `d23/d25/d32`, non-author `FIT` + `APPROVE`, and GitHub CI all pass.

## Stop and decide if

Stop if non-author reviewer finds `PREP.md` `NOT_FIT` or any file outside `antigravity-5/**` is modified.
